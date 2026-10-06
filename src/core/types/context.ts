/**
 * OCTREX CODE V4 - Context Engine & Repository Intelligence Contracts
 */

export interface SymbolReference {
  symbolName: string;
  kind: 'function' | 'class' | 'interface' | 'type' | 'variable';
  filePath: string;
  line: number;
  snippet: string;
}

export interface ContextSnippet {
  filePath: string;
  startLine: number;
  endLine: number;
  content: string;
  reason: string;
}

export interface PromptBudget {
  maxTotalTokens: number;
  systemTokensBudget: number;
  memoryTokensBudget: number;
  codeContextTokensBudget: number;
  toolOutputTokensBudget: number;
}

export interface ProjectMemory {
  framework?: string;
  buildCommands: string[];
  testCommands: string[];
  conventions: string[];
  knownIssues: string[];
}
