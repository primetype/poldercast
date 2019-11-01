use crate::{GossipsBuilder, NodeProfile, Nodes, ViewBuilder};

pub trait Layer {
    /// alias to the layer, for information purpose
    ///
    fn alias(&self) -> &'static str;

    /// reset the layer state
    fn reset(&mut self);

    /// populate the `Layer` based on all the nodes.
    /// The [`NodeProfile`] is given as the identity of the local node, allowing access
    /// to the local [`PublicId`] or [`Subscriptions`] (for example, used in the [`Rings`]
    /// or [`Vicinity`] module).
    ///
    /// [`NodeProfile`]: ./struct.NodeProfile.html
    /// [`PublicId`]: ./struct.PublicId.html
    /// [`Subscriptions`]: ./struct.Subscriptions.html
    /// [`Rings`]: ./poldercast/struct.Rings.html
    /// [`Vicinity`]: ./poldercast/struct.Vicinity.html
    fn populate(&mut self, identity: &NodeProfile, all_nodes: &Nodes);

    /// based on the populated layer, provide nodes to the [`ViewBuilder`].
    fn view(&mut self, view: &mut ViewBuilder, all_nodes: &mut Nodes);

    /// update the [`GossipsBuilder`], effectively preparing the list of gossips
    /// to share with the recipient (see [`GossipsBuilder`]). To build the gossips
    /// the `Layer` has access to the whole list of known [`Node`]s.
    ///
    /// [`GossipsBuilder`]: ./struct.GossipsBuilder.html
    /// [`Node`]: ./struct.Node.html
    fn gossips(&mut self, identity: &NodeProfile, gossips: &mut GossipsBuilder, all_nodes: &Nodes);
}
