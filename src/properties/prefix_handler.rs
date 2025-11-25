#![allow(non_snake_case)]
use super::{Property, PropertyId};
use crate::context::PropertyHandlerContext;
use crate::declaration::DeclarationList;
use crate::prefixes::Feature;
use crate::traits::{FallbackValues, IsCompatible, PropertyHandler};
use crate::vendor_prefix::VendorPrefix;

macro_rules! define_prefixes {
  (
    $( $name: ident, )+
  ) => {
    #[derive(Default)]
    pub(crate) struct PrefixHandler {
      $(
        $name: Option<usize>,
      )+
    }

    impl<'i> PropertyHandler<'i> for PrefixHandler {
      fn handle_property(&mut self, property: &Property<'i>, dest: &mut DeclarationList<'i>, context: &mut PropertyHandlerContext) -> bool {
        match property {
          $(
            Property::$name(val, prefix) => {
              // Get the expanded prefixes based on targets.
              let mut new_prefixes = context.targets.prefixes(*prefix, Feature::$name);
              
              // Scan through all existing properties of this type in dest
              // Determine which prefixes to add and which to remove from existing properties
              for i in 0..dest.len() {
                if let Property::$name(ref cur, ref mut existing_prefixes) = dest[i] {
                  if val == cur {
                    // Same value - merge the prefixes into existing property
                    *existing_prefixes |= new_prefixes;
                    *existing_prefixes = context.targets.prefixes(*existing_prefixes, Feature::$name);
                    // Update the tracked index to the merged property
                    self.$name = Some(i);
                    return true;
                  } else {
                    // Different value - check for overlapping prefixes
                    let overlap = *existing_prefixes & new_prefixes;
                    if !overlap.is_empty() {
                      // If the input prefix is None (unprefixed), don't override existing prefixed properties
                      // Otherwise, remove the overlapping prefixes from existing and keep them in new
                      if *prefix == VendorPrefix::None {
                        // Unprefixed property should not override explicit vendor prefixes
                        new_prefixes = new_prefixes.difference(overlap);
                      } else {
                        // Explicit vendor prefix overrides previous declarations
                        *existing_prefixes = existing_prefixes.difference(overlap);
                      }
                    }
                  }
                }
              }

              // If no prefixes remain after removing overlaps, skip this property
              if new_prefixes.is_empty() {
                return true;
              }

              // Store the index of the new property
              self.$name = Some(dest.len());
              dest.push(Property::$name(val.clone(), new_prefixes))
            }
          )+
          _ => return false
        }

        true
      }

      fn finalize(&mut self, _: &mut DeclarationList, _: &mut PropertyHandlerContext) {}
    }
  };
}

define_prefixes! {
  TransformOrigin,
  TransformStyle,
  BackfaceVisibility,
  Perspective,
  PerspectiveOrigin,
  BoxSizing,
  TabSize,
  Hyphens,
  TextAlignLast,
  TextDecorationSkipInk,
  TextOverflow,
  UserSelect,
  Appearance,
  ClipPath,
  BoxDecorationBreak,
  TextSizeAdjust,
}

macro_rules! define_fallbacks {
  (
    $( $name: ident$(($p: ident))?, )+
  ) => {
    pastey::paste! {
      #[derive(Default)]
      pub(crate) struct FallbackHandler {
        $(
          [<$name:snake>]: Option<usize>
        ),+
      }
    }

    impl<'i> PropertyHandler<'i> for FallbackHandler {
      fn handle_property(&mut self, property: &Property<'i>, dest: &mut DeclarationList<'i>, context: &mut PropertyHandlerContext<'i, '_>) -> bool {
        match property {
          $(
            Property::$name(val $(, mut $p)?) => {
              let mut val = val.clone();
              $(
                $p = context.targets.prefixes($p, Feature::$name);
              )?
              if pastey::paste! { self.[<$name:snake>] }.is_none() {
                let fallbacks = val.get_fallbacks(context.targets);
                #[allow(unused_variables)]
                let has_fallbacks = !fallbacks.is_empty();
                for fallback in fallbacks {
                  dest.push(Property::$name(fallback $(, $p)?))
                }

                $(
                  if has_fallbacks && $p.contains(VendorPrefix::None) {
                    $p = VendorPrefix::None;
                  }
                )?
              }

              if pastey::paste! { self.[<$name:snake>] }.is_none() || matches!(context.targets.browsers, Some(targets) if !val.is_compatible(targets)) {
                pastey::paste! { self.[<$name:snake>] = Some(dest.len()) };
                dest.push(Property::$name(val $(, $p)?));
              } else if let Some(index) = pastey::paste! { self.[<$name:snake>] } {
                dest[index] = Property::$name(val $(, $p)?);
              }
            }
          )+
          Property::Unparsed(val) => {
            let (mut unparsed, index) = match val.property_id {
              $(
                PropertyId::$name$(($p))? => {
                  macro_rules! get_prefixed {
                    ($vp: ident) => {
                      if $vp.contains(VendorPrefix::None) {
                        val.get_prefixed(context.targets, Feature::$name)
                      } else {
                        val.clone()
                      }
                    };
                    () => {
                      val.clone()
                    };
                  }

                  let val = get_prefixed!($($p)?);
                  (val, pastey::paste! { &mut self.[<$name:snake>] })
                }
              )+
              _ => return false
            };

            // Unparsed properties are always "valid", meaning they always override previous declarations.
            context.add_unparsed_fallbacks(&mut unparsed);
            if let Some(index) = *index {
              dest[index] = Property::Unparsed(unparsed);
            } else {
              *index = Some(dest.len());
              dest.push(Property::Unparsed(unparsed));
            }
          }
          _ => return false
        }

        true
      }

      fn finalize(&mut self, _: &mut DeclarationList, _: &mut PropertyHandlerContext) {
        $(
          pastey::paste! { self.[<$name:snake>] = None };
        )+
      }
    }
  };
}

define_fallbacks! {
  Color,
  TextShadow,
  Filter(prefix),
  BackdropFilter(prefix),
  Fill,
  Stroke,
  CaretColor,
  Caret,
}
