import * as fs from 'node:fs';
import * as path from 'node:path';
import { ProjectMemory } from '../types/context.js';

export class ProjectMemoryStore {
  private workspacePath: string;
  private memoryFilePath: string;
  private memory: ProjectMemory;

  constructor(workspacePath: string) {
    this.workspacePath = path.resolve(workspacePath);
    this.memoryFilePath = path.join(this.workspacePath, '.octrex', 'memory.json');
    this.memory = this.loadMemory();
  }

  private loadMemory(): ProjectMemory {
    try {
      if (fs.existsSync(this.memoryFilePath)) {
        const raw = fs.readFileSync(this.memoryFilePath, 'utf8');
        return JSON.parse(raw);
      }
    } catch {}

    return {
      framework: 'TypeScript Node ESM',
      buildCommands: ['npm run build'],
      testCommands: ['npm test'],
      conventions: ['Strict TypeScript', 'Zero-leak secret policy', 'Evidence-based verification'],
      knownIssues: [],
    };
  }

  public getMemory(): ProjectMemory {
    return { ...this.memory };
  }

  public updateMemory(partial: Partial<ProjectMemory>): void {
    this.memory = {
      ...this.memory,
      ...partial,
      conventions: Array.from(new Set([...(this.memory.conventions || []), ...(partial.conventions || [])])),
      knownIssues: Array.from(new Set([...(this.memory.knownIssues || []), ...(partial.knownIssues || [])])),
    };
    this.saveMemory();
  }

  public addConvention(convention: string): void {
    if (!this.memory.conventions.includes(convention)) {
      this.memory.conventions.push(convention);
      this.saveMemory();
    }
  }

  public addKnownIssue(issue: string): void {
    if (!this.memory.knownIssues.includes(issue)) {
      this.memory.knownIssues.push(issue);
      this.saveMemory();
    }
  }

  private saveMemory(): void {
    try {
      const dir = path.dirname(this.memoryFilePath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }
      fs.writeFileSync(this.memoryFilePath, JSON.stringify(this.memory, null, 2), 'utf8');
    } catch {}
  }
}
