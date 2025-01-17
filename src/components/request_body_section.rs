use anathema::{
    component::Component,
    state::{CommonVal, State},
};

pub const REQUEST_BODY_SECTION_TEMPLATE: &str =
    include_str!("./templates/request_body_section.aml");

#[derive(Default)]
pub struct RequestBodySection;

#[derive(Default, State)]
pub struct RequestBodySectionState {}

impl Component for RequestBodySection {
    type State = RequestBodySectionState;
    type Message = ();

    fn accept_focus(&self) -> bool {
        false
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        _state: &mut Self::State,
        mut elements: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        if let "request_body_border" = ident {
            let focus = value.to_bool();
            if focus {
                return;
            }

            let Some(border_color) = context.get_external("border_color") else {
                return;
            };
            let Some(border_color) = border_color.to_common() else {
                return;
            };

            // NOTE: Is this right?
            let color = border_color.to_string().leak();
            elements
                .by_attribute("id", "request_body_border")
                .each(|_element, attributes| {
                    attributes.set("foreground", CommonVal::Str(color));
                });
        }
    }
}
