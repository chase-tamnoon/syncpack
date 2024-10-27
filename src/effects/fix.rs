use {
  super::ui::Ui,
  crate::{
    context::Context,
    instance_state::{FixableInstance, InstanceState, InvalidInstance, SuspectInstance},
  },
  colored::*,
  log::{info, warn},
};

/// Run the fix command side effects
pub fn run(ctx: Context) -> Context {
  // @TODO: move values to config file
  let ui = Ui {
    ctx: &ctx,
    show_ignored: false,
    show_instances: true,
    show_local_hint: true,
    show_status_codes: true,
    show_packages: false,
    // @TODO: show_valid: false,
    // @TODO: sort_by: "name" | "state" | "count",
  };

  if ctx.config.cli.options.versions {
    ui.print_command_header("SEMVER RANGES AND VERSION MISMATCHES");
    let mut valid = 0;
    let mut fixable = 0;
    let mut unfixable = 0;
    let mut suspect = 0;

    ctx.instances.iter().for_each(|instance| {
      let name = &instance.name;
      let location = ui.instance_location(instance).dimmed();
      let state = instance.state.borrow().clone();
      let state_name = state.get_name();
      let state_link = ui.status_code_link(&state_name);
      let state_link = format!("({state_link})").dimmed();

      match state {
        InstanceState::Unknown => {}
        InstanceState::Valid(variant) => {
          valid += 1;
        }
        InstanceState::Invalid(variant) => match variant {
          InvalidInstance::Fixable(variant) => {
            fixable += 1;
            match variant {
              FixableInstance::IsBanned => instance.remove(),
              _ => {
                let actual = instance.actual_specifier.unwrap().red();
                let arrow = ui.dim_right_arrow();
                let expected = instance.expected_specifier.borrow().as_ref().unwrap().unwrap().green();
                info!("{name} {actual} {arrow} {expected} {location} {state_link}");
                instance.package.borrow().copy_expected_specifier(instance);
              }
            }
          }
          InvalidInstance::Conflict(_) | InvalidInstance::Unfixable(_) => {
            unfixable += 1;
            warn!("Unfixable: {name} {location} {state_link}");
          }
        },
        InstanceState::Suspect(variant) => match variant {
          SuspectInstance::RefuseToBanLocal
          | SuspectInstance::RefuseToPinLocal
          | SuspectInstance::RefuseToSnapLocal
          | SuspectInstance::InvalidLocalVersion => {
            suspect += 1;
            warn!("Suspect: {name} {location} {state_link}");
          }
        },
      }
    });

    info!("{} {} Already Valid", ui.count_column(valid), ui.ok_icon());
    info!("{} {} Fixed", ui.count_column(fixable), ui.ok_icon());
    info!("{} {} Unfixable", ui.count_column(unfixable), ui.err_icon());
    info!("{} {} Suspect", ui.count_column(suspect), ui.warn_icon());
  }

  if ctx.config.cli.options.format {
    ui.print_command_header("PACKAGE FORMATTING");
    let mut valid = 0;
    let mut fixable = 0;

    ctx.packages.all.iter().for_each(|package| {
      let package = package.borrow();
      let formatting_mismatches = package.formatting_mismatches.borrow();
      if formatting_mismatches.is_empty() {
        valid += 1;
      } else {
        fixable += 1;
        formatting_mismatches.iter().for_each(|mismatch| {
          if mismatch.property_path == "/" {
            *package.contents.borrow_mut() = mismatch.expected.clone();
          } else if let Some(value) = package.contents.borrow_mut().pointer_mut(&mismatch.property_path) {
            *value = mismatch.expected.clone();
          }
        });
      }
    });

    info!("{} {} Already Valid", ui.count_column(valid), ui.ok_icon());
    info!("{} {} Fixed", ui.count_column(fixable), ui.ok_icon());
  }

  ctx.packages.all.iter().for_each(|package| {
    package.borrow().write_to_disk(&ctx.config);
  });

  ctx
}
