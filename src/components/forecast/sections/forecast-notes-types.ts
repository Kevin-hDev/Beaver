export interface ForecastNote {
  id: string;
  analysis_id: string;
  date: string;
  title: string;
  note_type: string;
  source: "user" | "llm";
  content: string;
  file_path: string;
  created_at: string;
  updated_at: string;
}

export interface ForecastNotesAnalysis {
  name: string;
  target_column: string;
  model: string;
  horizon: number;
  input_summary: { start: string; end: string };
  predictions: { date: string; value: number }[];
}

export interface ForecastNoteDraft {
  id?: string;
  date: string;
  title: string;
  note_type: string;
  content: string;
}

export interface ForecastNotesProps {
  analysisId: string;
}

export interface NoteRange {
  start: number;
  end: number;
}
