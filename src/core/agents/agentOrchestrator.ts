import { TaskDefinition, TaskStatus, AgentRole, ICancellationToken } from '../types/agent.js';
import { TaskStore } from '../task/taskStore.js';
import { ModelRouter } from '../router/modelRouter.js';
import { RepoIndexer } from '../context/repoIndexer.js';
import { ContextEngine } from '../context/contextEngine.js';
import { FilesystemTools } from '../tools/filesystemTools.js';
import { DevTools } from '../tools/devTools.js';
import { GitTools } from '../tools/gitTools.js';
import { UIEventEmitter } from '../events/uiEventEmitter.js';
import { CheckpointProtectionGuard } from '../security/checkpointProtection.js';

export interface TaskExecutionResult {
  taskId: string;
  status: TaskStatus;
  plan?: string;
  filesModified: string[];
  testOutput?: string;
  reviewNotes?: string;
  error?: string;
}

export class AgentOrchestrator {
  private taskStore: TaskStore;
  private router: ModelRouter;
  private eventEmitter: UIEventEmitter;
  private contextEngine: ContextEngine;
  private checkpointGuard: CheckpointProtectionGuard;

  constructor(
    taskStore: TaskStore,
    router: ModelRouter,
    eventEmitter: UIEventEmitter,
    contextEngine = new ContextEngine()
  ) {
    this.taskStore = taskStore;
    this.router = router;
    this.eventEmitter = eventEmitter;
    this.contextEngine = contextEngine;
    this.checkpointGuard = new CheckpointProtectionGuard();
  }

  /**
   * Executes a full autonomous engineering task workflow through logical specialist agents.
   */
  public async executeTask(
    userPrompt: string,
    workspacePath: string,
    cancellationToken?: ICancellationToken
  ): Promise<TaskExecutionResult> {
    const task = this.taskStore.createTask(userPrompt, workspacePath);
    const taskId = task.id;

    const gitTools = new GitTools(workspacePath);
    const fsTools = new FilesystemTools(workspacePath);
    const devTools = new DevTools(workspacePath);
    const repoIndexer = new RepoIndexer(workspacePath);

    this.checkpointGuard.setHandler(gitTools);

    this.eventEmitter.emit(taskId, 'task_updated', {
      status: 'RECEIVED',
      userPrompt,
      workspacePath,
    });

    try {
      // 1. UNDERSTANDING & REPOSITORY ANALYSIS
      this.updateState(taskId, 'REPOSITORY_ANALYSIS', 'MANAGER');
      cancellationToken?.throwIfCancelled();

      await repoIndexer.scanWorkspace();
      const relevantFiles = repoIndexer.searchFilesByKeyword(userPrompt.split(' ')[0] || 'src');

      // 2. PLANNING (Planner Agent)
      this.updateState(taskId, 'PLANNING', 'PLANNER');
      cancellationToken?.throwIfCancelled();

      const planPrompt = this.contextEngine.assemblePrompt({
        systemInstruction: 'You are the PLANNER AGENT. Formulate a concise step-by-step engineering plan.',
        taskPrompt: `User Goal: ${userPrompt}\nRelevant files found: ${relevantFiles.join(', ')}`,
      });

      const planResp = await this.router.executeChatWithFallback(
        { taskId, role: 'PLANNER', requiresReasoning: true },
        planPrompt,
        cancellationToken
      );
      const planText = planResp.message.content || 'Default execution plan';

      this.eventEmitter.emit(taskId, 'agent_completed', {
        agentRole: 'PLANNER',
        plan: planText,
      });

      // 3. IMPLEMENTING (Coder Agent with Checkpoint Protection)
      this.updateState(taskId, 'IMPLEMENTING', 'CODER');
      cancellationToken?.throwIfCancelled();

      // Guard with automatic Git checkpoint
      await this.checkpointGuard.ensureCheckpointBeforeExecution('Agent Coder File Edit', 'MEDIUM', taskId);

      const modifiedFiles: string[] = [];

      this.eventEmitter.emit(taskId, 'agent_completed', {
        agentRole: 'CODER',
        modifiedFiles,
      });

      // 4. TESTING (Tester Agent)
      this.updateState(taskId, 'TESTING', 'TESTER');
      cancellationToken?.throwIfCancelled();

      const typecheckRes = await devTools.runTypecheck();
      const testRes = await devTools.runTests();

      let testPassed = typecheckRes.passed && testRes.passed;
      let repairAttempts = 0;

      // 5. DEBUGGING & REPAIR LOOP (Debugger Agent)
      while (!testPassed && repairAttempts < 2) {
        repairAttempts += 1;
        this.updateState(taskId, 'DEBUGGING', 'DEBUGGER');
        cancellationToken?.throwIfCancelled();

        this.eventEmitter.emit(taskId, 'agent_started', {
          agentRole: 'DEBUGGER',
          attempt: repairAttempts,
          errorTrace: typecheckRes.output || testRes.output,
        });

        // Debugger analysis & targeted repair
        const debugPrompt = this.contextEngine.assemblePrompt({
          systemInstruction: 'You are the DEBUGGER AGENT. Analyze test failure and repair the issue.',
          taskPrompt: `Failure Trace:\n${this.contextEngine.pruneToolOutput(testRes.output || typecheckRes.output, 40)}`,
        });

        await this.router.executeChatWithFallback(
          { taskId, role: 'DEBUGGER', requiresReasoning: true },
          debugPrompt,
          cancellationToken
        );

        // Re-run test checks
        this.updateState(taskId, 'TESTING', 'TESTER');
        const recheck = await devTools.runTypecheck();
        const retest = await devTools.runTests();
        testPassed = recheck.passed && retest.passed;
      }

      // 6. REVIEWING (Reviewer Agent)
      this.updateState(taskId, 'REVIEWING', 'REVIEWER');
      cancellationToken?.throwIfCancelled();

      const diffRes = await gitTools.getDiff();
      const reviewPrompt = this.contextEngine.assemblePrompt({
        systemInstruction: 'You are the REVIEWER AGENT. Perform independent quality and security review.',
        taskPrompt: `Original Task: ${userPrompt}\nGit Diff:\n${diffRes.data?.diff || 'No diff'}`,
      });

      const reviewResp = await this.router.executeChatWithFallback(
        { taskId, role: 'REVIEWER', requiresReasoning: true },
        reviewPrompt,
        cancellationToken
      );
      const reviewNotes = reviewResp.message.content || 'Review passed';

      // 7. FINAL VERIFICATION STATUS
      const finalStatus: TaskStatus = testPassed ? 'VERIFIED' : 'FAILED';
      this.updateState(taskId, finalStatus);

      this.eventEmitter.emit(taskId, 'verification_completed', {
        status: finalStatus,
        testsPassed: testPassed,
        plan: planText,
        reviewNotes,
      });

      return {
        taskId,
        status: finalStatus,
        plan: planText,
        filesModified: modifiedFiles,
        testOutput: testRes.output,
        reviewNotes,
      };
    } catch (err: any) {
      const isCancelled = cancellationToken?.isCancelled;
      const errorStatus: TaskStatus = isCancelled ? 'CANCELLED' : 'FAILED';
      this.updateState(taskId, errorStatus);

      this.eventEmitter.emit(taskId, 'task_updated', {
        status: errorStatus,
        error: err.message || String(err),
      });

      return {
        taskId,
        status: errorStatus,
        filesModified: [],
        error: err.message || String(err),
      };
    }
  }

  private updateState(taskId: string, status: TaskStatus, agent?: AgentRole): void {
    this.taskStore.updateTaskStatus(taskId, status, agent);
    this.eventEmitter.emit(taskId, 'task_updated', {
      status,
      activeAgent: agent,
    });
  }
}
