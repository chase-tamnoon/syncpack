extern crate serde;
extern crate serde_json;

mod dependencies;
mod read_file;

fn main() -> std::io::Result<()> {
    let files: Vec<&str> = vec![
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/apollo-content/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/apollo-error/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/atoms/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/commit-prompt/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/cookies/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/experiments/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/journey-composer/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/journey-tools/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/koa-masthead/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/molecules/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/organisms/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/page-speed-metrics/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/release-bot/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/selectors/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/sky-tags-events/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/stockpile/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/templates/package.json",
        "/Users/foldleft/Dev/sky-uk/pages-lib/packages/tracking/package.json",
    ];

    for file in files {
        println!("{}", file);
        let package = read_file::read_package_json(file)?;
        let dependencies = dependencies::get_dependencies("dependencies", &package);
        let dev_dependencies = dependencies::get_dependencies("devDependencies", &package);
        let peer_dependencies = dependencies::get_dependencies("peerDependencies", &package);
        println!("    dependencies");
        println!("        {:?}", dependencies);
        println!("    dev_dependencies");
        println!("        {:?}", dev_dependencies);
        println!("    peer_dependencies");
        println!("        {:?}", peer_dependencies);
    }

    Ok(())
}
