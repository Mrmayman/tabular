use iced::advanced::renderer;
use iced::Element;

pub mod tabular;
pub use tabular::*;

pub fn tabular<'a, Data, T, K, R, Message, Theme, Renderer>(
    tabular: &'a Data,
    view_cell: impl Fn(K, &'a T) -> Element<'a, Message, Theme, Renderer> + 'static,
) -> tabular::Table<'a, Data, T, K, R, Message, Theme, Renderer>
where
    Data: tabular::Tabular<T, K, R>,
    T: tabular::Cell + Default + 'a,
    K: tabular::reference::Reference + 'a,
    R: tabular::reference::ReferenceRange<K> + 'a,
    Message: Clone + 'a,
    Theme: tabular::Catalog + 'a,
    Renderer: renderer::Renderer + 'a,
{
    Table::new(tabular, view_cell)
}
