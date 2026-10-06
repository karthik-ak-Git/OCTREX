import * as fs from 'node:fs';
import * as path from 'node:path';
import { ExecutionResult, FilePatchOptions, FileReadOptions } from '../types/tools.js';

export class FilesystemTools {
  private workspacePath: string;

  constructor(workspacePath: string) {
    this.workspacePath = path.resolve(workspacePath);
  }

  private resolvePath(targetPath: string): string {
    if (path.isAbsolute(targetPath)) {
      return path.normalize(targetPath);
    }
    return path.normalize(path.join(this.workspacePath, targetPath));
  }

  public async readFile(options: FileReadOptions): Promise<ExecutionResult<{ content: string; totalLines: number }>> {
    const startTime = Date.now();
    try {
      const fullPath = this.resolvePath(options.filePath);
      if (!fs.existsSync(fullPath)) {
        return { success: false, error: `File not found: ${options.filePath}`, executionTimeMs: Date.now() - startTime };
      }

      const raw = fs.readFileSync(fullPath, 'utf8');
      const lines = raw.split('\n');
      const start = Math.max(1, options.startLine || 1);
      const end = Math.min(lines.length, options.endLine || lines.length);

      const slice = lines.slice(start - 1, end).join('\n');
      return {
        success: true,
        data: { content: slice, totalLines: lines.length },
        executionTimeMs: Date.now() - startTime,
      };
    } catch (err: any) {
      return { success: false, error: err.message, executionTimeMs: Date.now() - startTime };
    }
  }

  public async writeFile(filePath: string, content: string): Promise<ExecutionResult<{ bytesWritten: number }>> {
    const startTime = Date.now();
    try {
      const fullPath = this.resolvePath(filePath);
      const dir = path.dirname(fullPath);
      if (!fs.existsSync(dir)) {
        fs.mkdirSync(dir, { recursive: true });
      }

      fs.writeFileSync(fullPath, content, 'utf8');
      return {
        success: true,
        data: { bytesWritten: Buffer.byteLength(content, 'utf8') },
        executionTimeMs: Date.now() - startTime,
      };
    } catch (err: any) {
      return { success: false, error: err.message, executionTimeMs: Date.now() - startTime };
    }
  }

  public async patchFile(options: FilePatchOptions): Promise<ExecutionResult<{ linesReplaced: number }>> {
    const startTime = Date.now();
    try {
      const fullPath = this.resolvePath(options.filePath);
      if (!fs.existsSync(fullPath)) {
        return { success: false, error: `File not found: ${options.filePath}`, executionTimeMs: Date.now() - startTime };
      }

      const content = fs.readFileSync(fullPath, 'utf8');
      const lines = content.split('\n');

      const start = Math.max(1, options.startLine);
      const end = Math.min(lines.length, options.endLine);

      if (start > end || start > lines.length) {
        return { success: false, error: `Invalid line range ${start}-${end} (Total lines: ${lines.length})`, executionTimeMs: Date.now() - startTime };
      }

      const replacementLines = options.replacementContent.split('\n');
      lines.splice(start - 1, end - start + 1, ...replacementLines);

      fs.writeFileSync(fullPath, lines.join('\n'), 'utf8');
      return {
        success: true,
        data: { linesReplaced: end - start + 1 },
        executionTimeMs: Date.now() - startTime,
      };
    } catch (err: any) {
      return { success: false, error: err.message, executionTimeMs: Date.now() - startTime };
    }
  }

  public async deleteFile(filePath: string): Promise<ExecutionResult> {
    const startTime = Date.now();
    try {
      const fullPath = this.resolvePath(filePath);
      if (fs.existsSync(fullPath)) {
        const stat = fs.statSync(fullPath);
        if (stat.isDirectory()) {
          fs.rmSync(fullPath, { recursive: true, force: true });
        } else {
          fs.unlinkSync(fullPath);
        }
      }
      return { success: true, executionTimeMs: Date.now() - startTime };
    } catch (err: any) {
      return { success: false, error: err.message, executionTimeMs: Date.now() - startTime };
    }
  }
}
