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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.measure {
        let palette = tmux_palette::palettes::load_builtin(&args.palette)
            .ok_or_else(|| anyhow::anyhow!("Unknown palette: {}", args.palette))?;
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
        let desired = items.len() + cats + 7;
        let rows = desired.min(28);
        println!("{rows}\t90\t3\tnone\tdefault\tdefault");
        return Ok(());
    }

    anyhow::bail!("Rust TUI is not implemented yet; use the TypeScript launcher for now")
}
