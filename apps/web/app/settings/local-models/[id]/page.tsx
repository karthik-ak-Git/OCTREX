import React from 'react';
import LocalModelDetailClient from './LocalModelDetailClient';

export function generateStaticParams() {
  // Static-export requires at least one route; the client resolves any model id at runtime.
  return [{ id: 'placeholder' }];
}

export const dynamicParams = false;

export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <LocalModelDetailClient modelId={decodeURIComponent(id)} />;
}
