import * as Icon from "@/components/svg/icons";

export function getIconFromName(name: string, size?: number) {
  switch (name.toLowerCase()) {
    case "autocomplete":
    case "cli completions":
      return <Icon.Autocomplete size={size} />;
    case "integrations":
      return <Icon.Apps size={size} />;
    case "keybindings":
      return <Icon.Keybindings size={size} />;
    case "preferences":
    default:
      return <Icon.Settings size={size} />;
  }
}
