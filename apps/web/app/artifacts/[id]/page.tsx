import React from 'react';
import { ArtifactDetailClient } from './ArtifactDetailClient';

export function generateStaticParams() {
  return [{ id: 'placeholder' }];
}

export const dynamicParams = false;

export default async function Page({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  return <ArtifactDetailClient artifactId={decodeURIComponent(id)} />;
}
