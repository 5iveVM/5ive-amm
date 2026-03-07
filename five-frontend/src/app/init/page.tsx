import type { Metadata } from "next";
import InitWorkflowPage from "@/components/init/InitWorkflowPage";

export const metadata: Metadata = {
  title: "5ive Init Workflow | 5IVE",
  description:
    "Start with 5ive init, shape AGENTS.md, generate projects in a few shots, and deploy with a practical AI-assisted workflow.",
};

export default function InitPage() {
  return <InitWorkflowPage />;
}
