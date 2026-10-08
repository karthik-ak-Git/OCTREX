import React from 'react';
import { DocumentDetailClient } from './DocumentDetailClient';

export function generateStaticParams() {
  return [{ id: 'placeholder' }];
}

export const dynamicParams = false;

export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <DocumentDetailClient documentId={decodeURIComponent(id)} />;
}
