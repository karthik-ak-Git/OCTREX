import React, { useState, useEffect } from 'react';
import { Sidebar } from './components/Sidebar';
import { ChatCanvas } from './components/ChatCanvas';
import { InspectorPanel, InspectorTab } from './components/InspectorPanel';
import { OnboardingModal } from './components/OnboardingModal';
import { SettingsHub, SettingsHubTab } from './components/SettingsHub';
import { useOctrexStore } from './state';

export const App: React.FC = () => {
  const store = useOctrexStore();

  // Modals & Inspector Navigation
  const [isOnboardingOpen, setIsOnboardingOpen] = useState(false);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [settingsInitialTab, setSettingsInitialTab] = useState<SettingsHubTab>('sessions');

  const [inspectorTab, setInspectorTab] = useState<InspectorTab>('activity');
  const [openPreviews, setOpenPreviews] = useState<
    Array<{ id: string; name: string; type: 'document' | 'image' }>
  >([
    { id: 'p1', name: 'roadmap-q4.pptx', type: 'document' },
    { id: 'p2', name: 'brand-guide.png', type: 'image' }
  ]);

  // First launch onboarding check
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

  const handleOpenSettings = (tab: SettingsHubTab = 'sessions') => {
    setSettingsInitialTab(tab);
    setIsSettingsOpen(true);
  };

  const handleClosePreview = (tabType: InspectorTab) => {
    setOpenPreviews((prev) => prev.filter((p) => p.type !== tabType));
    if (inspectorTab === tabType) {
      setInspectorTab('activity');
    }
  };

  const handleOpenProjectFolder = async () => {
    try {
      if (window.octrex?.openProject) {
        const res = await window.octrex.openProject();
        if (res && res.path) {
          const newProj = store.addProject(res.name, res.path);
          store.createNewSession(newProj.id);
          return;
        }
      }
    } catch (err) {
      console.warn('Native open project failed, fallback:', err);
    }
    // Fallback: prompt for directory name
    const folderName = prompt('Enter project folder name:', 'my-awesome-app');
    if (folderName) {
      const newProj = store.addProject(folderName, `~/dev/${folderName}`);
      store.createNewSession(newProj.id);
    }
  };

  const handleOpenFilePreview = (fileName: string) => {
    if (fileName.endsWith('.png') || fileName.endsWith('.jpg') || fileName.endsWith('.svg')) {
      if (!openPreviews.some((p) => p.type === 'image')) {
        setOpenPreviews((prev) => [
          ...prev,
          { id: `img-${Date.now()}`, name: fileName, type: 'image' }
        ]);
      }
      setInspectorTab('image');
    } else {
      if (!openPreviews.some((p) => p.type === 'document')) {
        setOpenPreviews((prev) => [
          ...prev,
          { id: `doc-${Date.now()}`, name: fileName, type: 'document' }
        ]);
      }
      setInspectorTab('document');
    }
  };

  return (
    <div className="flex h-screen w-screen bg-[#f4f6f8] text-slate-900 overflow-hidden font-sans select-none antialiased relative">
      {/* Ambient background blur spheres for rich glassmorphism depth */}
      <div className="ambient-glow ambient-cyan w-[500px] h-[500px] top-10 left-48 opacity-25" />
      <div className="ambient-glow ambient-indigo w-[600px] h-[600px] -bottom-32 right-64 opacity-20" />

      {/* 1. Left Sidebar Navigation */}
      <Sidebar
        projects={store.projects}
        activeWorkspaceId={store.activeWorkspaceId}
        onSelectWorkspace={(wsId) => store.setActiveWorkspaceId(wsId)}
        activeSessionId={store.activeSessionId}
        sessions={store.sessions}
        onSelectSession={(id) => store.setActiveSessionId(id)}
        onNewChat={() => store.createNewSession()}
        onOpenProject={handleOpenProjectFolder}
        onOpenSettings={handleOpenSettings}
      />

      {/* 2. Center Chat & Task Canvas */}
      <ChatCanvas
        activeSession={store.activeSession}
        skills={store.skills}
        onSelectModel={(model) => store.updateModel(model)}
        onSendMessage={(prompt) => store.sendMessage(prompt)}
        onRespondPermission={(msgId, decision, scope) =>
          store.respondPermission(msgId, decision, scope)
        }
        onOpenFilePreview={handleOpenFilePreview}
        onOpenTerminal={() => setInspectorTab('terminal')}
        onOpenSkillsSettings={() => handleOpenSettings('skills')}
      />

      {/* 3. Right Inspector & Previews Panel */}
      <InspectorPanel
        activeTab={inspectorTab}
        onTabChange={(tab) => setInspectorTab(tab)}
        onClosePreview={handleClosePreview}
        openPreviews={openPreviews}
        activeSession={store.activeSession}
        tasks={store.tasks}
        onStopTask={(tId) => store.stopTask(tId)}
        onAskEdit={(instruction) => store.sendMessage(instruction)}
      />

      {/* 4. Onboarding Modal Carousel */}
      <OnboardingModal
        isOpen={isOnboardingOpen}
        onClose={() => setIsOnboardingOpen(false)}
        onFinish={handleFinishOnboarding}
        onOpenFolder={handleOpenProjectFolder}
        onSelectWorkspace={(name) => {
          if (name) {
            const match = store.projects.find((p) => p.name === name);
            if (match) store.setActiveWorkspaceId(match.id);
          }
        }}
        onConnectProvider={store.connectProvider}
        recentWorkspaces={store.projects.map((p) => ({ name: p.name, path: p.path }))}
      />

      {/* 5. Full Settings Hub */}
      <SettingsHub
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
        initialTab={settingsInitialTab}
        sessions={store.sessions}
        onSelectSession={(id) => store.setActiveSessionId(id)}
        onDeleteSession={(id) => store.deleteSession(id)}
        skills={store.skills}
        onToggleSkill={(id) => store.toggleSkill(id)}
        onAddSkill={(name, desc, prompt) => store.addSkill(name, desc, prompt)}
        mcpServers={store.mcpServers}
        onToggleMcpServer={(id) => store.toggleMcpServer(id)}
        onAddMcpServer={(name, url, trans) => store.addMcpServer(name, url, trans)}
        connectedProviders={store.connectedProviders}
        onConnectProvider={store.connectProvider}
      />
    </div>
  );
};
export default App;
