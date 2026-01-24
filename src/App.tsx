import { useState } from "react";
import { MainLayout } from "./components/layout/MainLayout";
import { ConnectionForm } from "./components/ConnectionForm";
import { SyncPage } from "./components/SyncPage";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { useConnections } from "./hooks/useConnections";
import { Connection, ConnectionInput } from "./lib/api";

function App() {
  const { connections, saveConnection, deleteConnection, testConnection } = useConnections();
  const [formOpen, setFormOpen] = useState(false);
  const [editingConnection, setEditingConnection] = useState<Connection | null>(null);

  const handleNewConnection = () => {
    setEditingConnection(null);
    setFormOpen(true);
  };

  const handleEditConnection = (id: string) => {
    const conn = connections.find((c) => c.id === id);
    if (conn) {
      setEditingConnection(conn);
      setFormOpen(true);
    }
  };

  const handleDeleteConnection = async (id: string) => {
    await deleteConnection(id);
  };

  const handleSave = async (input: ConnectionInput) => {
    await saveConnection(input);
  };

  const handleTest = async (input: ConnectionInput) => {
    await testConnection(input);
  };

  return (
    <ErrorBoundary>
      <MainLayout
        connections={connections}
        onNewConnection={handleNewConnection}
        onEditConnection={handleEditConnection}
        onDeleteConnection={handleDeleteConnection}
      >
        <ErrorBoundary>
          <SyncPage connections={connections} />
        </ErrorBoundary>
        <ConnectionForm
          open={formOpen}
          onOpenChange={setFormOpen}
          connection={editingConnection}
          onSave={handleSave}
          onTest={handleTest}
        />
      </MainLayout>
    </ErrorBoundary>
  );
}

export default App;
