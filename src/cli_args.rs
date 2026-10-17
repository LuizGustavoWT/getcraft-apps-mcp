//! Locate native subcommands without confusing argument values with servers.
//!
//! FilmCraft v0.8.0 and EffectCraft v0.10.0 accept options anywhere. Their
//! argument parsers at the commits in upstream-versions.json use these valued
//! options; the other five apps dispatch on the first argument.

const FILM_VALUES: &[&str] = &[
    "--project",
    "--save-as",
    "--bridge",
    "--format",
    "--scale",
    "--quality",
    "--seconds",
    "--out",
    "--frames",
    "--preset",
    "--range",
    "--start",
    "--end",
    "--data-dir",
    "--settings",
    "--gpu-rendering",
];

const EFFECT_VALUES: &[&str] = &[
    "--n",
    "--play",
    "--layers",
    "--comps",
    "--footage",
    "--params",
    "--project",
    "--bridge",
    "--save-as",
    "--filter",
    "--time",
    "--frame",
    "--comp",
    "--max-side",
    "--scale",
    "--out",
    "--expression",
    "--depth",
    "--eval",
    "--template",
    "--values",
    "--format",
    "--start",
    "--end",
    "--fps",
    "--resolution",
    "--quality",
    "--channels",
    "--jpeg-quality",
    "--bitrate",
    "--prores",
    "--audio",
    "--profile",
    "--level",
    "--rate-control",
    "--video-quality",
    "--keyint",
    "--webm-codec",
    "--audio-bitrate",
    "--opus-app",
    "--automation-read-root",
    "--automation-write-root",
];

pub fn subcommand<'a>(app: &str, args: &'a [String]) -> Option<&'a str> {
    let valued = match app {
        "filmcraft" => FILM_VALUES,
        "effectcraft" => EFFECT_VALUES,
        _ => return args.first().map(String::as_str),
    };
    let mut positionals = Vec::new();
    let mut explicit_project = false;
    let mut arguments = args.iter();
    while let Some(argument) = arguments.next() {
        if app == "filmcraft" && argument == "--" {
            positionals.extend(arguments.map(String::as_str));
            break;
        }
        if argument.starts_with("--") && argument.len() > 2 {
            let (key, inline) = argument
                .split_once('=')
                .map_or((argument.as_str(), false), |(key, _)| (key, true));
            explicit_project |= key == "--project";
            if !inline && valued.contains(&key) {
                arguments.next();
            }
        } else {
            positionals.push(argument.as_str());
        }
    }
    // EffectCraft removes the first positional project anywhere in the list,
    // unless --project was supplied (even after the subcommand).
    if app == "effectcraft"
        && !explicit_project
        && let Some(index) = positionals
            .iter()
            .position(|value| value.ends_with(".ecproj"))
    {
        positionals.remove(index);
    }
    positionals.first().copied()
}
