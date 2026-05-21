import { useTranslation } from "react-i18next";
import { CheckCircle2 } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardFooter, CardHeader } from "@/components/ui/Card";

interface Props {
  onGoToInbox: () => void;
}

export function SuccessStep({ onGoToInbox }: Props) {
  const { t } = useTranslation();

  return (
    <Card className="w-full">
      <CardHeader className="items-center text-center">
        <CheckCircle2 className="h-12 w-12 text-green-500" />
        <h1 className="text-2xl font-semibold">{t("onboarding.success")}</h1>
      </CardHeader>
      <CardContent />
      <CardFooter className="flex-col items-stretch">
        <Button size="lg" className="w-full" onClick={onGoToInbox}>
          {t("onboarding.go_to_inbox")}
        </Button>
      </CardFooter>
    </Card>
  );
}
