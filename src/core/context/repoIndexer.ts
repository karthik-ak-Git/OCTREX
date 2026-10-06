import * as fs from 'node:fs';
import * as path from 'node:path';
import { SymbolReference } from '../types/context.js';

export interface FileIndexEntry {
  filePath: string;
  relativePath: string;
  sizeBytes: number;
  lastModified: number;
  extension: string;
  isTestFile: boolean;
}

export class RepoIndexer {
  private workspacePath: string;
  private indexedFiles: Map<string, FileIndexEntry> = new Map();
  private symbolIndex: Map<string, SymbolReference[]> = new Map();

  constructor(workspacePath: string) {
    this.workspacePath = path.resolve(workspacePath);
  }

  public async scanWorkspace(ignoreDirs: string[] = ['node_modules', '.git', 'dist', 'build', '.octrex']): Promise<number> {
    this.indexedFiles.clear();
    this.symbolIndex.clear();

    const walk = (dir: string) => {
      let entries: fs.Dirent[];
      try {
        entries = fs.readdirSync(dir, { withFileTypes: true });
      } catch {
        return;
      }

      for (const entry of entries) {
        if (ignoreDirs.includes(entry.name)) continue;

        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.isFile()) {
          const relPath = path.relative(this.workspacePath, fullPath).replace(/\\/g, '/');
          const ext = path.extname(entry.name).toLowerCase();
          const stat = fs.statSync(fullPath);
          const isTest = relPath.includes('.test.') || relPath.includes('.spec.') || relPath.includes('/tests/') || relPath.includes('/test/');

          this.indexedFiles.set(relPath, {
            filePath: fullPath,
            relativePath: relPath,
            sizeBytes: stat.size,
            lastModified: stat.mtimeMs,
            extension: ext,
            isTestFile: isTest,
          });

          // Index symbols if code file
          if (['.ts', '.js', '.tsx', '.jsx', '.py', '.rs', '.go', '.java'].includes(ext)) {
            this.indexSymbolsInFile(fullPath, relPath);
          }
        }
      }
    };

    walk(this.workspacePath);
    return this.indexedFiles.size;
  }

  private indexSymbolsInFile(fullPath: string, relPath: string): void {
    try {
      const content = fs.readFileSync(fullPath, 'utf8');
      const lines = content.split('\n');

      const symbolRegexes = [
        { kind: 'function' as const, regex: /(?:function\s+([a-zA-Z0-9_$]+)|const\s+([a-zA-Z0-9_$]+)\s*=\s*(?:async\s*)?\([^)]*\)\s*=>)/ },
        { kind: 'class' as const, regex: /class\s+([a-zA-Z0-9_$]+)/ },
        { kind: 'interface' as const, regex: /interface\s+([a-zA-Z0-9_$]+)/ },
        { kind: 'type' as const, regex: /type\s+([a-zA-Z0-9_$]+)\s*=/ },
      ];

      lines.forEach((line, idx) => {
        const lineNum = idx + 1;
        for (const { kind, regex } of symbolRegexes) {
          const match = line.match(regex);
          if (match) {
            const symName = match[1] || match[2];
            if (symName) {
              const ref: SymbolReference = {
                symbolName: symName,
                kind,
                filePath: relPath,
                line: lineNum,
                snippet: line.trim(),
              };

              let list = this.symbolIndex.get(symName);
              if (!list) {
                list = [];
                this.symbolIndex.set(symName, list);
              }
              list.push(ref);
            }
          }
        }
      });
    } catch {
      // Skip unreadable files
    }
  }

  public findSymbols(nameQuery: string): SymbolReference[] {
    const results: SymbolReference[] = [];
    const lowerQuery = nameQuery.toLowerCase();

    for (const [sym, refs] of this.symbolIndex.entries()) {
      if (sym.toLowerCase().includes(lowerQuery)) {
        results.push(...refs);
      }
    }

    return results;
  }

  public findRelatedTests(sourceRelPath: string): string[] {
    const baseName = path.basename(sourceRelPath, path.extname(sourceRelPath)).toLowerCase();
    const testFiles: string[] = [];

    for (const [relPath, entry] of this.indexedFiles.entries()) {
      if (entry.isTestFile) {
        const lowerRel = relPath.toLowerCase();
        if (lowerRel.includes(baseName) || (baseName.includes('gateway') && lowerRel.includes('gateway'))) {
          testFiles.push(relPath);
        }
      }
    }

    return testFiles;
  }

  public searchFilesByKeyword(keyword: string): string[] {
    const matchedFiles: string[] = [];
    const lower = keyword.toLowerCase();

    for (const [relPath, entry] of this.indexedFiles.entries()) {
      if (relPath.toLowerCase().includes(lower)) {
        matchedFiles.push(relPath);
        continue;
      }
      try {
        const content = fs.readFileSync(entry.filePath, 'utf8');
        if (content.toLowerCase().includes(lower)) {
          matchedFiles.push(relPath);
        }
      } catch {}
    }

    return matchedFiles;
  }

  public getIndexedFiles(): FileIndexEntry[] {
    return Array.from(this.indexedFiles.values());
  }
}
