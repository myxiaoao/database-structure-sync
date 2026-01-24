import { MainLayout } from './components/layout/MainLayout';

function App() {
  return (
    <MainLayout>
      <div className="text-center">
        <h1 className="text-2xl font-bold">Database Structure Sync</h1>
        <p className="text-muted-foreground mt-2">Select connections to compare</p>
      </div>
    </MainLayout>
  );
}

export default App;
