import React from 'react';
import { DocumentSearchClient } from './DocumentSearchClient';

export function generateStaticParams() {
  return [{ id: 'placeholder' }];
}

export const dynamicParams = false;

export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <DocumentSearchClient documentId={decodeURIComponent(id)} />;
}
