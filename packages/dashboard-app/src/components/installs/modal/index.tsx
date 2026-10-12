import { InstallCheck } from "@/types/preferences";
import { useEffect, useState } from "react";
import { Button } from "../../ui/button";
import Lockup from "../../svg/logo";
import onboarding from "@/data/onboarding";
import { useStatusCheck } from "@/hooks/store/useStatusCheck";
import InstallModal from "./install";
import { useLocalState } from "@/hooks/store/useState";

export default function OnboardingModal() {
  const [step, setStep] = useState(0);
  const check = onboarding[step] as InstallCheck | undefined;
  const [, setCompletedOnboarding] = useLocalState(
    "desktop.completedOnboarding",
  );
  const [, refreshDotfiles] = useStatusCheck("dotfiles");
  const [, refreshAccessibility] = useStatusCheck("accessibility");
  const [, refreshInputMethod] = useStatusCheck("inputMethod");

  useEffect(() => {
    refreshAccessibility();
    refreshDotfiles();
    refreshInputMethod();
  }, [refreshAccessibility, refreshDotfiles, refreshInputMethod]);

  function nextStep() {
    if (step + 1 >= onboarding.length) {
      setCompletedOnboarding(true);
    } else {
      setStep(step + 1);
    }
  }

  if (!check) return null;

  if (check.id === "welcome") {
    return <WelcomeModal next={nextStep} />;
  }

  return <InstallModal check={check} skip={nextStep} next={nextStep} />;
}

export function WelcomeModal({ next }: { next: () => void }) {
  return (
    <div className="flex flex-col items-center gap-8">
      <div className="flex flex-col items-center gap-6">
        <Lockup />
        <p className="text-sm text-zinc-500 dark:text-zinc-400 select-none">
          Let's get you set up...
        </p>
      </div>
      <Button onClick={() => next()} className="dark:bg-dusk-600">
        Get started
      </Button>
    </div>
  );
}
