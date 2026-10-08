import React from 'react';
import TaskVerificationClient from './TaskVerificationClient';

export function generateStaticParams() {
  // Static-export requires at least one route; the client resolves any task id at runtime.
  return [{ id: 'placeholder' }];
}

export const dynamicParams = false;

export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <TaskVerificationClient taskId={decodeURIComponent(id)} />;
}
