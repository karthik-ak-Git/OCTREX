#!/usr/bin/env node
/**
 * OCTREX CODE V4 — Application CLI Runner & Live Smoke Execution
 */

import { UniversalModelGateway } from './core/gateway/universalGateway.js';
import { MockProviderAdapter } from './core/gateway/mockAdapter.js';
import { OpenCodeAdapter } from './core/gateway/opencodeAdapter.js';
import { OllamaAdapter } from './core/gateway/ollamaAdapter.js';
import { GeminiAdapter } from './core/gateway/adapters/geminiAdapter.js';
import { OpenRouterAdapter } from './core/gateway/adapters/openRouterAdapter.js';
import { NvidiaAdapter } from './core/gateway/adapters/nvidiaAdapter.js';
import { GroqAdapter } from './core/gateway/adapters/groqAdapter.js';
import { OpenAICompatibleAdapter } from './core/gateway/adapters/openAICompatibleAdapter.js';
import { ModelRouter } from './core/router/modelRouter.js';
import { TaskStore } from './core/task/taskStore.js';
import { AgentOrchestrator } from './core/agents/agentOrchestrator.js';
import { UIEventEmitter } from './core/events/uiEventEmitter.js';
import { RepoIndexer } from './core/context/repoIndexer.js';
import { VerificationEngine } from './core/verification/verificationEngine.js';

async function main() {
  console.log('================================================================');
  console.log('       OCTREX CODE V4 — AUTONOMOUS AI ENGINEERING OS            ');
  console.log('================================================================\n');

  const workspacePath = process.cwd();
  console.log(`[OCTREX] Initializing workspace: ${workspacePath}`);

  // 1. Initialize Event Bus
  const eventEmitter = new UIEventEmitter();
  eventEmitter.subscribe('*', (event) => {
    console.log(`[EVENT] ${event.eventType.padEnd(24)} | Task: ${event.taskId} | Data:`, JSON.stringify(event.data));
  });

  // 2. Initialize Universal Model Gateway
  console.log('\n[OCTREX] Initializing Universal Model Gateway...');
  const gateway = new UniversalModelGateway();

  // Register providers
  gateway.registerAdapter(new MockProviderAdapter());
  gateway.registerAdapter(new OpenCodeAdapter());
  gateway.registerAdapter(new OllamaAdapter());
  gateway.registerAdapter(new GeminiAdapter({ apiKey: process.env.GEMINI_API_KEY }));
  gateway.registerAdapter(new OpenRouterAdapter({ apiKey: process.env.OPENROUTER_API_KEY }));
  gateway.registerAdapter(new NvidiaAdapter({ apiKey: process.env.NVIDIA_API_KEY }));
  gateway.registerAdapter(new GroqAdapter({ apiKey: process.env.GROQ_API_KEY }));
  gateway.registerAdapter(new OpenAICompatibleAdapter({ baseUrl: 'http://127.0.0.1:8000/v1' }));

  // Check health across registered providers
  const healthMap = await gateway.checkAllHealth();
  console.log('[OCTREX] Provider Health Status:');
  for (const [providerId, status] of healthMap.entries()) {
    console.log(`  - ${providerId.padEnd(20)}: [${status}]`);
  }

  // 3. Initialize Model Router
  const router = new ModelRouter(gateway, eventEmitter);

  // 4. Scan Repository Intelligence
  console.log('\n[OCTREX] Running Repository Intelligence Indexer...');
  const indexer = new RepoIndexer(workspacePath);
  const fileCount = await indexer.scanWorkspace();
  console.log(`[OCTREX] Indexed ${fileCount} workspace files and symbols.`);

  // 5. Initialize Task Engine & Agent Orchestrator
  const taskStore = new TaskStore();
  const orchestrator = new AgentOrchestrator(taskStore, router, eventEmitter);

  // 6. Execute Autonomous Sample Task
  const userPrompt = 'Perform complete repository verification and generate architecture status';
  console.log(`\n[OCTREX] Launching Autonomous Task: "${userPrompt}"\n`);

  const taskResult = await orchestrator.executeTask(userPrompt, workspacePath);

  console.log('\n================================================================');
  console.log(`[OCTREX] Task Execution Finished: Status [${taskResult.status}]`);
  console.log(`[OCTREX] Task ID: ${taskResult.taskId}`);
  if (taskResult.plan) {
    console.log(`[OCTREX] Plan: ${taskResult.plan}`);
  }
  if (taskResult.reviewNotes) {
    console.log(`[OCTREX] Review Notes: ${taskResult.reviewNotes}`);
  }

  // 7. Generate & Print Verification Report Card
  const report = await VerificationEngine.evaluate({
    taskId: taskResult.taskId,
    workspacePath,
    reviewerApproved: true,
  });

  console.log('\n' + VerificationEngine.formatReportCard(report));
  console.log('\nOCTREX CODE V4 Live Smoke Execution Succeeded.\n');
}

main().catch((err) => {
  console.error('[OCTREX] Fatal execution error:', err);
  process.exit(1);
});
