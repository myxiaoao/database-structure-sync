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
      <div className="h-0.5 bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 shrink-0" />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar
          connections={connections}
          onNewConnection={onNewConnection}
          onEditConnection={onEditConnection}
          onDeleteConnection={onDeleteConnection}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          <header className="flex items-center justify-between px-5 border-b h-[52px] shrink-0">
            <div className="flex items-center gap-3">
              <AppLogo size={26} />
              <div>
                <h1 className="font-semibold text-base leading-tight">{t("app.title")}</h1>
                <p className="text-[11px] text-muted-foreground leading-tight">
                  {t("sync.headerDesc")}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-1.5">
              <LanguageToggle />
              <ThemeToggle />
            </div>
          </header>
          <main className="flex-1 overflow-hidden">{children}</main>
        </div>
      </div>
    </div>
  );
}
