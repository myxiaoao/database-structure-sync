import { ReactNode } from 'react';
import { Sidebar } from './Sidebar';
import { ThemeToggle } from '@/components/ThemeToggle';
import { LanguageToggle } from '@/components/LanguageToggle';
import { Connection } from '@/lib/api';

interface MainLayoutProps {
  children: ReactNode;
  connections?: Connection[];
  onNewConnection?: () => void;
  onEditConnection?: (id: string) => void;
  onDeleteConnection?: (id: string) => void;
}

export function MainLayout({
  children,
  connections = [],
  onNewConnection,
  onEditConnection,
  onDeleteConnection,
}: MainLayoutProps) {
  return (
    <div className="flex h-screen bg-background">
      <Sidebar
        connections={connections}
        onNewConnection={onNewConnection}
        onEditConnection={onEditConnection}
        onDeleteConnection={onDeleteConnection}
      />
      <div className="flex-1 flex flex-col overflow-hidden">
        <header className="flex items-center justify-end gap-2 px-4 py-2 border-b">
          <LanguageToggle />
          <ThemeToggle />
        </header>
        <main className="flex-1 overflow-auto p-6">{children}</main>
      </div>
    </div>
  );
}
