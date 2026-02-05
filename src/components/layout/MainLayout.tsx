import { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Sidebar } from "./Sidebar";
import { ThemeToggle } from "@/components/ThemeToggle";
import { LanguageToggle } from "@/components/LanguageToggle";
import { AppLogo } from "@/components/AppLogo";
import { Connection } from "@/lib/api";

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
  const { t } = useTranslation();

  return (
    <div className="flex flex-col h-screen bg-background">
      {/* 顶部装饰线 */}
      <div className="h-0.5 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 shrink-0" />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          connections={connections}
          onNewConnection={onNewConnection}
          onEditConnection={onEditConnection}
          onDeleteConnection={onDeleteConnection}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          <header className="flex items-center justify-between px-4 py-2 border-b h-[45px]">
            <div className="flex items-center gap-3">
              <AppLogo size={28} />
              <span className="font-semibold text-lg">{t("app.title")}</span>
            </div>
            <div className="flex items-center gap-2">
              <LanguageToggle />
              <ThemeToggle />
            </div>
          </header>
          <main className="flex-1 overflow-auto p-4">{children}</main>
        </div>
      </div>
    </div>
  );
}
