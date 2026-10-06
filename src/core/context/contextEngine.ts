import { ContextSnippet, PromptBudget, ProjectMemory } from '../types/context.js';
import { ChatMessage } from '../types/gateway.js';

export class ContextEngine {
  private defaultBudget: PromptBudget = {
    maxTotalTokens: 128000,
    systemTokensBudget: 8000,
    memoryTokensBudget: 8000,
    codeContextTokensBudget: 80000,
    toolOutputTokensBudget: 32000,
  };

  /**
   * Approximates token count (approx 4 characters per token).
   */
  public estimateTokens(text: string): number {
    return Math.ceil((text || '').length / 4);
  }

  /**
   * Truncates long terminal or tool outputs keeping critical head and error tail.
   */
  public pruneToolOutput(rawOutput: string, maxLines = 100): string {
    if (!rawOutput) return '';
    const lines = rawOutput.split('\n');
    if (lines.length <= maxLines) return rawOutput;

    const head = lines.slice(0, 25);
    const tail = lines.slice(-75);
    return [...head, `\n... [${lines.length - 100} lines truncated to conserve context] ...\n`, ...tail].join('\n');
  }

  /**
   * Compacts older conversational turns into a concise structured memory summary.
   */
  public compactConversation(messages: ChatMessage[], maxTurns = 6): ChatMessage[] {
    if (messages.length <= maxTurns) {
      return messages;
    }

    const systemMessages = messages.filter((m) => m.role === 'system');
    const recentMessages = messages.slice(-maxTurns);
    const olderMessages = messages.slice(0, messages.length - maxTurns).filter((m) => m.role !== 'system');

    // Summarize older turns
    const summaryLines = olderMessages.map((m) => {
      const preview = m.content.slice(0, 120).replace(/\n/g, ' ');
      return `- [${m.role.toUpperCase()}]: ${preview}...`;
    });

    const compactionMessage: ChatMessage = {
      role: 'system',
      content: `[Previous Context Summary - ${olderMessages.length} turns compacted]:\n${summaryLines.join('\n')}`,
    };

    return [...systemMessages, compactionMessage, ...recentMessages];
  }

  /**
   * Assembles a structured, budgeted prompt payload.
   */
  public assemblePrompt(params: {
    systemInstruction?: string;
    taskPrompt: string;
    memory?: ProjectMemory;
    snippets?: ContextSnippet[];
    conversation?: ChatMessage[];
  }): ChatMessage[] {
    const messages: ChatMessage[] = [];

    // 1. System Instruction & Rules
    if (params.systemInstruction) {
      messages.push({
        role: 'system',
        content: params.systemInstruction,
      });
    }

    // 2. Project Memory & Conventions
    if (params.memory) {
      const memLines: string[] = [];
      if (params.memory.framework) memLines.push(`Framework: ${params.memory.framework}`);
      if (params.memory.buildCommands?.length) memLines.push(`Build Commands: ${params.memory.buildCommands.join(', ')}`);
      if (params.memory.testCommands?.length) memLines.push(`Test Commands: ${params.memory.testCommands.join(', ')}`);
      if (params.memory.conventions?.length) memLines.push(`Conventions: ${params.memory.conventions.join('; ')}`);

      if (memLines.length > 0) {
        messages.push({
          role: 'system',
          content: `[Project Memory & Verified Rules]:\n${memLines.join('\n')}`,
        });
      }
    }

    // 3. Relevant Code Context Snippets
    if (params.snippets && params.snippets.length > 0) {
      const snippetBlocks = params.snippets.map(
        (s) => `--- File: ${s.filePath} (Lines ${s.startLine}-${s.endLine}) [${s.reason}] ---\n${s.content}`
      );

      messages.push({
        role: 'system',
        content: `[Relevant Repository Code Context]:\n${snippetBlocks.join('\n\n')}`,
      });
    }

    // 4. Conversation Turns (Compacted if long)
    if (params.conversation && params.conversation.length > 0) {
      const compacted = this.compactConversation(params.conversation);
      messages.push(...compacted.filter((m) => m.role !== 'system'));
    }

    // 5. Active Task Request
    messages.push({
      role: 'user',
      content: params.taskPrompt,
    });

    return messages;
  }
}
