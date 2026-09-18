import { Layout } from './components/layout/Layout';
import { useAppStore } from './stores/useAppStore';
import { DashboardView } from './features/dashboard/DashboardView';
import { DownloadsView } from './features/downloads/DownloadsView';
import { BatchesView } from './features/batches/BatchesView';
import { CompletedView } from './features/completed/CompletedView';
import { FailedView } from './features/failed/FailedView';
import { ProfilesView } from './features/profiles/ProfilesView';
import { SettingsView } from './features/settings/SettingsView';
import { AboutView } from './features/about/AboutView';
import { ImportModal } from './features/imports/ImportModal';
import './styles/globals.css';

export function App() {
  const { activeTab } = useAppStore();

  const renderActiveView = () => {
    switch (activeTab) {
      case 'dashboard':
        return <DashboardView />;
      case 'downloads':
        return <DownloadsView />;
      case 'batches':
        return <BatchesView />;
      case 'completed':
        return <CompletedView />;
      case 'failed':
        return <FailedView />;
      case 'profiles':
        return <ProfilesView />;
      case 'settings':
        return <SettingsView />;
      case 'about':
        return <AboutView />;
      default:
        return <DashboardView />;
    }
  };

  return (
    <Layout>
      {renderActiveView()}
      <ImportModal />
    </Layout>
  );
}

export default App;
