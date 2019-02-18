# Poldercast's P2P topology organization

This crate implements the Poldercast's Peer to Peer (P2P) topology construction.
The idea is to allow the node to participate actively into building the decentralized
topology of the p2p network.

This is done through gossiping. This is the process of sharing with others topology
information: who is on the network, how to reach them and what are they interested
about.

In the poldercast paper there are 3 different modules implementing 3 different strategies
to select nodes to gossip to and to select the gossiping data:

* `Cyclon`: this module is responsible to add a bit of randomness in the gossiping
  strategy. It also prevent nodes to be left behind, favouring contacting Nodes
  we have the least used;
* `Vicinity`: this module helps with building an interest-induced links between the
  nodes of the topology. Making sure that nodes that have common interests are often
  in touch.
* `Rings`: this module create an oriented list of nodes. It is an arbitrary way to
  link the nodes in the network. For each topics, the node will select a set of _close_
  nodes (see documentation in the implementation for more details about this).

# Papers of reference

This crate is a concrete implementation of the Poldercast paper:

* [PolderCast: Fast, Robust, and Scalable Architecture for P2P Topic-Based Pub/Sub](https://hal.inria.fr/hal-01555561/document)
* [CYCLON: Inexpensive Membership Management for Unstructured P2P Overlays](http://www.csl.mtu.edu/cs6461/www/Reading/Voulgaris-jnsm05.pdf)
* [Vicinity: Epidemic-Based Self-Organization in Peer-To-Peer Systems](https://www.cs.vu.nl/~spyros/papers/Thesis-Voulgaris.pdf)

# Customization

Now this crate allows room for different kind of management of the modules.
It is possible to add the Poldercast default modules (the default). It is
possible to setup a custom topology strategy utilizing part or all
of the poldercast's modules or with new custom modules.

# License

This project is licensed under either of the following licenses:

* Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

Please choose the licence you want to use.
