import { pushToast } from "$lib/toast";

export async function copyToClipboard(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    pushToast("Copied to clipboard");
  } catch {
    pushToast("Could not copy to clipboard", "error");
  }
}
