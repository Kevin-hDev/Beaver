export interface ModelParameter {
  key: string;
  value: string;
}

export interface OllamaModelEditorData {
  modelfile: string;
  parameters: ModelParameter[];
}
