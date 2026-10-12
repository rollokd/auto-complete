import { InstallCheckWithInstallKey } from "@/types/preferences";
import { PRODUCT_NAME } from "@/lib/constants";

const getBackupsDir = (): string => {
  const backupsDir = window?.fig?.constants?.backupsDir;
  const home = window?.fig?.constants?.home;
  if (backupsDir && home) {
    return backupsDir.replace(home, "~") + "/";
  } else {
    return "~/.auto.dotfiles.bak/";
  }
};

const installChecks: InstallCheckWithInstallKey[] = [
  {
    id: "dotfiles",
    installKey: "dotfiles",
    title: "Shell integration",
    description: [
      `Integrate ${PRODUCT_NAME} with zsh so it can run the necessary hooks.`,
    ],
    action: "Install",
    explainer: {
      title: "What's happening under the hood?",
      steps: [
        [
          {
            content: `${PRODUCT_NAME} will add one line to the top and bottom of the following files (if they exist):`,
            tag: "span",
          },
          {
            content: ".zshrc",
            tag: "code",
          },
          {
            content: ".zprofile",
            tag: "code",
          },
        ],
        [
          {
            content: "Your dotfiles will be backed up to",
            tag: "span",
          },
          {
            content: getBackupsDir(),
            tag: "code",
          },
        ],
        [
          {
            content: `Uninstalling ${PRODUCT_NAME} will remove these lines.`,
            tag: "span",
          },
        ],
      ],
    },
  },
  {
    id: "accessibility",
    installKey: "accessibility",
    title: "Enable accessibility",
    description: [
      `Grant accessibility permissions so ${PRODUCT_NAME} can position the completion window and insert text on your behalf.`,
      `If enabling it isn't working, try toggling it off and on again or restarting ${PRODUCT_NAME}.`,
    ],
    action: "Enable",
    actionWaitingText: "Waiting for permission...",
  },
  {
    id: "inputMethod",
    installKey: "inputMethod",
    title: "Install input method",
    description: [
      `Ghostty and Zed report the cursor position to ${PRODUCT_NAME} through a macOS input method.`,
      "Restart your terminals after installing. If the popup doesn't follow the cursor, log out and back in.",
    ],
    action: "Install",
  },
];

export default installChecks;
