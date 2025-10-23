import { Loader2 } from "lucide-react";
import type React from "react";

export function Spinner(props: { size: number; children?: React.ReactNode }) {
  return (
    <Loader2 size={props.size ?? 14} className={"animate-spin text-primary"} />
  );
}
