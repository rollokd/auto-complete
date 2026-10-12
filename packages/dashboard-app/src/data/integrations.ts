import { PrefSection } from "@/components/preference/list";
import { PRODUCT_NAME } from "@/lib/constants";

const integrationOptions: PrefSection[] = [
  {
    title: "Terminals",
    properties: [
      {
        id: "integrations.ghostty.disabled",
        title: "Ghostty",
        description: `Show ${PRODUCT_NAME} in Ghostty.`,
        type: "boolean",
        default: false,
        inverted: true,
      },
      {
        id: "integrations.zed.disabled",
        title: "Zed",
        description: `Show ${PRODUCT_NAME} in Zed's built-in terminal.`,
        type: "boolean",
        default: false,
        inverted: true,
      },
    ],
  },
];

export default integrationOptions;
