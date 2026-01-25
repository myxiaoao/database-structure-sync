import { MainLayout } from "./components/layout/MainLayout";
import { ConnectionForm } from "./components/ConnectionForm";
import { SyncPage } from "./components/SyncPage";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { useConnections } from "./hooks";

function App() {
  const {
    connections,
    editingConnection,
    isFormOpen,
    openNewConnection,
    openEditConnection,
    closeForm,
    saveConnection,
    deleteConnection,
    testConnection,
  } = useConnections();

  const handleEditConnection = (id: string) => {
    const conn = connections.find((c) => c.id === id);
    if (conn) {
      openEditConnection(conn);
    }
  };

  return (
    <ErrorBoundary>
      <MainLayout
        connections={connections}
        onNewConnection={openNewConnection}
        onEditConnection={handleEditConnection}
        onDeleteConnection={deleteConnection}
      >
        <ErrorBoundary>
          <SyncPage connections={connections} />
        </ErrorBoundary>
        <ConnectionForm
          open={isFormOpen}
          onOpenChange={(open) => !open && closeForm()}
          connection={editingConnection}
          onSave={saveConnection}
          onTest={testConnection}
        />
      </MainLayout>
    </ErrorBoundary>
  );
}

export default App;
