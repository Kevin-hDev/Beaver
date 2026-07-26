export interface ForecastDocSection {
  id: string;
  title: string;
  body: string;
}

export interface ForecastDocPage {
  id: string;
  navLabel: string;
  title: string;
  summary: string;
  sections: ForecastDocSection[];
}
