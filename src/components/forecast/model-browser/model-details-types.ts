export interface ForecastModelDetails {
  description_short: string;
  description_long_markdown: string;
  source_url: string;
  source_label: string;
  license?: string | null;
  pipeline_tag?: string | null;
  library_name?: string | null;
  downloads?: number | null;
  likes?: number | null;
  tags: string[];
}
