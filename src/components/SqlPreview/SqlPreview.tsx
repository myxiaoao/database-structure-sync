import { useTranslation } from 'react-i18next';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';

interface SqlPreviewProps {
  sql: string;
}

export function SqlPreview({ sql }: SqlPreviewProps) {
  const { t } = useTranslation();

  return (
    <Card className="h-full flex flex-col">
      <CardHeader className="py-3 px-4">
        <CardTitle className="text-sm">{t('sql.preview')}</CardTitle>
      </CardHeader>
      <CardContent className="flex-1 p-0">
        <ScrollArea className="h-full">
          {sql ? (
            <pre className="p-4 text-xs font-mono whitespace-pre-wrap break-all">
              {sql}
            </pre>
          ) : (
            <p className="p-4 text-sm text-muted-foreground">{t('sql.empty')}</p>
          )}
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
