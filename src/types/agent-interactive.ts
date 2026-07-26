export interface AgentInteractiveOption {
  id?: string;
  label: string;
  description: string;
  recommended?: boolean;
  preview?: string;
}

export interface AgentInteractiveQuestion {
  header: string;
  question: string;
  options: AgentInteractiveOption[];
  multiSelect?: boolean;
}

export interface AgentInteractiveChoiceRequest {
  sessionId: string;
  id: string;
  questions: AgentInteractiveQuestion[];
  currentIndex: number;
  total: number;
}

export interface AgentInteractiveAnswer {
  questionIndex: number;
  selectedIds: string[];
  selectedLabels: string[];
  customAnswer?: string;
}
