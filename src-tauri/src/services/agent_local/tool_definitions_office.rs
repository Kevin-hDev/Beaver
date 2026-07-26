use serde_json::Value;

fn tool_def(name: &str, description: &str, parameters: Value) -> Value {
    serde_json::json!({
        "type": "function",
        "function": { "name": name, "description": description, "parameters": parameters }
    })
}

pub fn office_tool_definitions() -> Vec<Value> {
    vec![
        tool_def(
            "read_spreadsheet",
            "Read data from a spreadsheet (Excel .xlsx/.xls/.ods or CSV/TSV). Relative paths resolve from the working directory. \
             Returns JSON with headers and rows. \
             The first row is always used as column headers. Formulas are returned as text (e.g. '=SUM(A1:A5)'), not computed values. \
             Always pass a relative path with its subdirectory (not a bare filename). \
             Example: {\"path\": \"data/sales.xlsx\", \"sheet\": \"Sheet1\", \"max_rows\": 100}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the file"},
                    "sheet": {"type": "string", "description": "Sheet name (default: first). Ignored for CSV."},
                    "range": {"type": "string", "description": "Cell range (e.g. 'A1:D10'). Ignored for CSV."},
                    "max_rows": {"type": "integer", "description": "Max data rows (default: 500, max: 5000)"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "read_document",
            "Extract text from PDF or Word (.docx). Relative paths resolve from the working directory. \
             Returns JSON with text content. \
             Always pass a relative path with its subdirectory (not a bare filename). \
             Example: {\"path\": \"docs/report.pdf\"}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to PDF or .docx file"},
                    "pages": {"type": "string", "description": "Page range for PDF (e.g. '1-5'). Optional."}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "read_image",
            "Read image metadata (dimensions, format, size). Relative paths resolve from the working directory. \
             Supports JPEG, PNG, WebP, GIF, BMP. \
             Always pass a relative path with its subdirectory (not a bare filename). \
             Example: {\"path\": \"assets/photo.jpg\"}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to the image file"}
                },
                "required": ["path"]
            }),
        ),
        tool_def(
            "write_spreadsheet",
            "Create or modify an Excel file (.xlsx). Relative paths resolve from the working directory. \
             Use operations array to set cells, formulas, rows, and formatting. \
             New files get a default sheet named 'Sheet1'. Each operation can target a specific sheet via 'sheet' (default: first sheet). \
             Do not use add_sheet for single-sheet files — Sheet1 is created automatically. \
             Formatting ops: set_format (bold/italic/underline/font_color/bg_color/font_size, optional 'value' to rewrite the cell), \
             set_number_format (number_format like '0.00', 'DD/MM/YYYY', '#,##0 €'), set_border (border_style: thin|medium|thick, border_sides: [top,bottom,left,right]), \
             merge_cells (start_cell + end_cell like 'A1' and 'C3'), set_row_height (row + height). \
             Always pass a relative path with its subdirectory (not a bare filename). \
             Example: {\"path\": \"output/report.xlsx\", \"operations\": [\
             {\"type\": \"set_row\", \"row\": 0, \"values\": [\"Name\", \"Age\"]}, \
             {\"type\": \"set_cell\", \"cell\": \"A2\", \"value\": \"Alice\"}, \
             {\"type\": \"set_format\", \"cell\": \"A1\", \"bold\": true, \"bg_color\": \"FF0000\"}, \
             {\"type\": \"merge_cells\", \"start_cell\": \"A1\", \"end_cell\": \"C1\"}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to .xlsx file"},
                    "operations": {
                        "type": "array",
                        "description": "Array of operations. Each can have 'sheet' to target a specific sheet (default: first).",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["set_cell", "set_formula", "set_row", "add_sheet", "set_column_width", "set_format", "set_number_format", "set_border", "merge_cells", "set_row_height"]},
                                "sheet": {"type": "string", "description": "Target sheet name (default: first sheet)"},
                                "cell": {"type": "string"},
                                "row": {"type": "integer"},
                                "col": {"type": "integer"},
                                "value": {"type": "string", "description": "Cell value. Pass numbers or booleans as text if needed."},
                                "formula": {"type": "string"},
                                "values": {"type": "array", "items": {"type": "string"}},
                                "name": {"type": "string"},
                                "width": {"type": "number"},
                                "bold": {"type": "boolean", "description": "For set_format"},
                                "italic": {"type": "boolean", "description": "For set_format"},
                                "underline": {"type": "boolean", "description": "For set_format"},
                                "font_color": {"type": "string", "description": "For set_format — hex RRGGBB (e.g. 'FF0000')"},
                                "bg_color": {"type": "string", "description": "For set_format — hex RRGGBB (e.g. 'FFFF00')"},
                                "font_size": {"type": "number", "description": "For set_format"},
                                "number_format": {"type": "string", "description": "For set_number_format (e.g. '0.00', 'DD/MM/YYYY', '#,##0 €')"},
                                "border_style": {"type": "string", "enum": ["thin", "medium", "thick"], "description": "For set_border"},
                                "border_sides": {"type": "array", "items": {"type": "string", "enum": ["top", "bottom", "left", "right"]}, "description": "For set_border — which sides get the border"},
                                "start_cell": {"type": "string", "description": "For merge_cells (e.g. 'A1')"},
                                "end_cell": {"type": "string", "description": "For merge_cells (e.g. 'C3')"},
                                "height": {"type": "number", "description": "For set_row_height"}
                            },
                            "required": ["type"]
                        }
                    }
                },
                "required": ["path", "operations"]
            }),
        ),
        tool_def(
            "write_document",
            "Create a Word document (.docx) from content blocks. Relative paths resolve from the working directory. \
             Only include fields relevant to each block type. \
             heading: {type, text, level, align?}. paragraph: {type, text, bold?, italic?, runs?, align?} — use 'runs' (array of {text, bold?, italic?, underline?, color?}) to mix styles within one paragraph. \
             table: {type, headers, rows}. list: {type, items, ordered}. \
             align (heading/paragraph): 'left'|'center'|'right'|'justify'. color: hex RRGGBB (e.g. 'FF0000'). \
             Always pass a relative path with its subdirectory (not a bare filename). \
             Example: {\"path\": \"docs/report.docx\", \"content\": [\
             {\"type\": \"heading\", \"text\": \"Title\", \"level\": 1, \"align\": \"center\"}, \
             {\"type\": \"paragraph\", \"runs\": [{\"text\": \"Hello \"}, {\"text\": \"world\", \"bold\": true, \"color\": \"FF0000\"}]}, \
             {\"type\": \"table\", \"headers\": [\"A\",\"B\"], \"rows\": [[\"1\",\"2\"]]}, \
             {\"type\": \"list\", \"items\": [\"a\",\"b\"], \"ordered\": true}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Output path for .docx file"},
                    "content": {
                        "type": "array",
                        "description": "Content blocks. Each block uses only the fields for its type.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["heading", "paragraph", "table", "list"]},
                                "text": {"type": "string", "description": "For heading/paragraph only"},
                                "level": {"type": "integer", "description": "For heading only (1-6)"},
                                "bold": {"type": "boolean", "description": "For paragraph only (when not using runs)"},
                                "italic": {"type": "boolean", "description": "For paragraph only (when not using runs)"},
                                "align": {"type": "string", "enum": ["left", "center", "right", "justify"], "description": "For heading/paragraph alignment"},
                                "runs": {
                                    "type": "array",
                                    "description": "For paragraph only — array of styled segments. Overrides 'text' when present.",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "text": {"type": "string"},
                                            "bold": {"type": "boolean"},
                                            "italic": {"type": "boolean"},
                                            "underline": {"type": "boolean"},
                                            "color": {"type": "string", "description": "hex RRGGBB (e.g. 'FF0000')"}
                                        },
                                        "required": ["text"]
                                    }
                                },
                                "headers": {"type": "array", "items": {"type": "string"}, "description": "For table only — column headers"},
                                "rows": {"type": "array", "items": {"type": "array", "items": {"type": "string"}}, "description": "For table only — array of arrays"},
                                "items": {"type": "array", "items": {"type": "string"}, "description": "For list only"},
                                "ordered": {"type": "boolean", "description": "For list only (true=numbered, false=bullets)"}
                            },
                            "required": ["type"]
                        }
                    }
                },
                "required": ["path", "content"]
            }),
        ),
        tool_def(
            "process_image",
            "Resize, crop, or convert an image. Relative paths resolve from the working directory. \
             To convert format, just change the extension. \
             Always pass a relative path with its subdirectory (not a bare filename). \
             Example convert: {\"input_path\": \"assets/photo.png\", \"output_path\": \"output/photo.webp\"} \
             Example resize: {\"input_path\": \"assets/photo.png\", \"output_path\": \"output/thumb.jpg\", \"operations\": [\
             {\"type\": \"resize\", \"width\": 200, \"height\": 200, \"mode\": \"fit\"}, \
             {\"type\": \"quality\", \"value\": 85}]}",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "input_path": {"type": "string", "description": "Source image path"},
                    "output_path": {"type": "string", "description": "Output path (extension determines format: jpg, png, webp, gif, bmp)"},
                    "operations": {
                        "type": "array",
                        "description": "Optional. Operations: resize ({type,width,height,mode:'fit'|'fill'|'exact'}), crop ({type,x,y,width,height}), quality ({type,value:1-100}). Omit for simple format conversion.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {"type": "string", "enum": ["resize", "crop", "quality"]},
                                "width": {"type": "integer"},
                                "height": {"type": "integer"},
                                "mode": {"type": "string", "enum": ["fit", "fill", "exact"]},
                                "x": {"type": "integer"},
                                "y": {"type": "integer"},
                                "value": {"type": "integer"}
                            },
                            "required": ["type"]
                        }
                    }
                },
                "required": ["input_path", "output_path"]
            }),
        ),
    ]
}
