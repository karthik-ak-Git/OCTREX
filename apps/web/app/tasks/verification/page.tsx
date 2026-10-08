'use client';

import React, { Suspense } from 'react';
import { useSearchParams } from 'next/navigation';
import TaskVerificationClient from '../[id]/verification/TaskVerificationClient';

function VerificationByQuery() {
  const params = useSearchParams();
  const taskId = params.get('taskId') ?? '';
  if (!taskId) {
    return (
      <div className="p-8 max-w-5xl mx-auto text-sm text-zinc-400">
        Missing <span className="font-mono">?taskId=…</span> query parameter. Open verification
        from a task page or provide a task ID.
      </div>
    );
  }
  return <TaskVerificationClient taskId={taskId} />;
}

/**
 * Static-export-compatible verification entry.
 * The canonical route is /tasks/[id]/verification; this static page serves
 * the same UI behind the static file server (which cannot prerender arbitrary
 * task IDs) via ?taskId=….
 */
export default function TaskVerificationQueryPage() {
  return (
    <Suspense fallback={<div className="p-8 text-xs text-zinc-500">Loading verification…</div>}>
      <VerificationByQuery />
    </Suspense>
  );
}
