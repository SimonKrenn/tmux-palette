use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "tmux-palette")]
struct Args {
    #[arg(default_value = "commands")]
    palette: String,
    #[arg(long)]
    measure: bool,
    #[arg(long)]
    cw: Option<u16>,
    #[arg(long)]
    ch: Option<u16>,
    #[arg(long)]
    category: Option<String>,
}

fn apply_category_filter(palette: &mut tmux_palette::model::PaletteDef, category: Option<&str>) {
    if let Some(category) = category {
        palette
            .items
            .retain(|item| item.category.as_deref() == Some(category));
        palette.title = Some(category.to_string());
        palette.grouped = false;
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.measure {
        let mut palette = tmux_palette::config::load_palette(&args.palette)
            .ok_or_else(|| anyhow::anyhow!("Unknown palette: {}", args.palette))?;
        apply_category_filter(&mut palette, args.category.as_deref());
        let items = palette.items;
        let grouped = palette.grouped;
        let cats = if grouped {
            let mut seen = std::collections::BTreeSet::new();
            for item in &items {
                if let Some(category) = &item.category {
                    seen.insert(category.clone());
                }
            }
            seen.len()
        } else {
            0
        };
        let sizing = tmux_palette::config::user_sizing();
        let max_height = sizing.max_height.unwrap_or(28) as usize;
        let width = sizing.width.unwrap_or(90);
        let mut pad_x = sizing.pad_x.unwrap_or(3);
        let mobile_width = sizing.mobile_width.unwrap_or(80);
        let border = sizing.border.unwrap_or_else(|| "none".into());
        let theme = tmux_palette::theme::resolve_active_theme(None)?;
        let body_style = sizing
            .body_style
            .unwrap_or_else(|| format!("bg={}", theme.panel));
        let border_style = sizing
            .border_style
            .unwrap_or_else(|| format!("fg={},bg=default", theme.accent));
        let desired = items.len() + cats + 7;
        let mut rows = desired.min(max_height) as u16;
        let mut final_width = width;
        if mobile_width > 0 && args.cw.is_some_and(|cw| cw < mobile_width) {
            rows = rows.max(args.ch.unwrap_or(rows));
            final_width = args.cw.unwrap_or(final_width);
            pad_x = 1;
        }
        println!("{rows}\t{final_width}\t{pad_x}\t{border}\t{body_style}\t{border_style}");
        return Ok(());
    }

    let mut palette = tmux_palette::config::load_palette(&args.palette)
        .ok_or_else(|| anyhow::anyhow!("Unknown palette: {}", args.palette))?;
    apply_category_filter(&mut palette, args.category.as_deref());
    tmux_palette::tui::run_palette(palette, args.palette)
}
