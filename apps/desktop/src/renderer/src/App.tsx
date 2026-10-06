import React, { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { ChatCanvas } from './components/ChatCanvas';
import { InspectorPanel, InspectorTab } from './components/InspectorPanel';
import { OnboardingModal } from './components/OnboardingModal';
import { SettingsHub } from './components/SettingsHub';

export const App: React.FC = () => {
  // Modal states
  const [isOnboardingOpen, setIsOnboardingOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settingsInitialTab, setSettingsInitialTab] = useState<
    'sessions' | 'skills' | 'mcp' | 'chats' | 'permissions'
  >('sessions');

  // Workspace and Session state
  const [activeWorkspace, setActiveWorkspace] = useState<string>('octrex-web');
  const [activeSessionId, setActiveSessionId] = useState<string>('session-1');
  const [activeModel, setActiveModel] = useState<string>('Claude Sonnet 3.5');

  // Inspector panel tab and open previews
  const [inspectorTab, setInspectorTab] = useState<InspectorTab>('activity');
  const [openPreviews, setOpenPreviews] = useState<
    Array<{ id: string; name: string; type: 'document' | 'image' }>
  >([
    { id: 'p1', name: 'roadmap-q4.pptx', type: 'document' },
    { id: 'p2', name: 'brand-guide.png', type: 'image' }
  ]);

  // Initial check for onboarding
  useEffect(() => {
    const hasCompletedOnboarding = localStorage.getItem('octrex_onboarded_v4');
    if (!hasCompletedOnboarding) {
      setIsOnboardingOpen(true);
    }
  }, []);

  const handleFinishOnboarding = () => {
    localStorage.setItem('octrex_onboarded_v4', 'true');
    setIsOnboardingOpen(false);
  };

  const handleOpenSettings = (
    tab: 'sessions' | 'skills' | 'mcp' | 'chats' | 'permissions' = 'sessions'
  ) => {
    setSettingsInitialTab(tab);
    setIsSettingsOpen(true);
  };

  const handleClosePreview = (tabType: InspectorTab) => {
    setOpenPreviews((prev) => prev.filter((p) => p.type !== tabType));
    if (inspectorTab === tabType) {
      setInspectorTab('activity');
    }
  };

  return (
    <div className="flex h-screen w-screen bg-[#f4f6f8] text-slate-900 overflow-hidden font-sans select-none antialiased">
      {/* 1. Left Sidebar Navigation */}
      <Sidebar
        activeWorkspace={activeWorkspace}
        onSelectWorkspace={(ws) => setActiveWorkspace(ws)}
        activeSessionId={activeSessionId}
        onSelectSession={(id) => setActiveSessionId(id)}
        onOpenSettings={handleOpenSettings}
        onNewChat={() => {
          const newId = `session-${Date.now()}`;
          setActiveSessionId(newId);
        }}
      />

      {/* 2. Center Chat & Task Canvas */}
      <ChatCanvas
        activeWorkspace={activeWorkspace}
        activeModel={activeModel}
        onSelectModel={(model) => setActiveModel(model)}
        onOpenFilePreview={(file) => {
          if (file.endsWith('.pptx') || file.endsWith('.pdf') || file.endsWith('.md')) {
            if (!openPreviews.some((p) => p.type === 'document')) {
              setOpenPreviews((prev) => [
                ...prev,
                { id: `doc-${Date.now()}`, name: file, type: 'document' }
              ]);
            }
            setInspectorTab('document');
          } else if (file.endsWith('.png') || file.endsWith('.jpg') || file.endsWith('.svg')) {
            if (!openPreviews.some((p) => p.type === 'image')) {
              setOpenPreviews((prev) => [
                ...prev,
                { id: `img-${Date.now()}`, name: file, type: 'image' }
              ]);
            }
            setInspectorTab('image');
          }
        }}
        onOpenTerminal={() => setInspectorTab('terminal')}
      />

      {/* 3. Right Inspector & Previews Panel */}
      <InspectorPanel
        activeTab={inspectorTab}
        onTabChange={(tab) => setInspectorTab(tab)}
        onClosePreview={handleClosePreview}
        openPreviews={openPreviews}
      />

      {/* 4. Onboarding Modal Carousel (Steps 1, 2, 3) */}
      <OnboardingModal
        isOpen={isOnboardingOpen}
        onClose={() => setIsOnboardingOpen(false)}
        onFinish={handleFinishOnboarding}
      />

      {/* 5. Full Settings Hub (Sessions, Skills, MCP, Chats, Permissions) */}
      <SettingsHub
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        initialTab={settingsInitialTab}
      />
    </div>
  );
};
export default App;
