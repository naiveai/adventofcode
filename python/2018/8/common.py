class Node:
    def __init__(self, child_nodes, metadata_entries):
        self.child_nodes = child_nodes
        self.metadata_entries = metadata_entries

        self.num_child_nodes_left = None
        self.num_metadata_entries_left = None

    @classmethod
    def empty(cls):
        return cls([], [])

    def add_child_node(self, node):
        self.child_nodes.append(node)
        self.num_child_nodes_left -= 1

    def add_metadata_entry(self, entry):
        self.metadata_entries.append(entry)
        self.num_metadata_entries_left -= 1

    def __repr__(self):
        return (f"Node(child_nodes={self.child_nodes}" +
                f", metadata_entries={self.metadata_entries})")


# My original solution that I came up with after way too much thought. Uses the
# concept of layers to keep every child node and metadata item straight. I
# thought it was excellent of me to observe that you are only ever reading a
# header or reading metadata when you're looking at a specific digit. But this
# didn't help me get into the heart of the question, and there's a far more
# elegant solution down below this one.
def parse_digits_complicated(digit_list):
    current_layer = 0

    layers = [Node.empty()]

    is_header_state = True
    header_on_num_child_nodes = True

    for digit in digit_list:
        if is_header_state:
            if header_on_num_child_nodes:
                layers[current_layer].num_child_nodes_left = digit
                header_on_num_child_nodes = False
            else:
                layers[current_layer].num_metadata_entries_left = digit

                if layers[current_layer].num_child_nodes_left:
                    current_layer += 1
                else:
                    is_header_state = False
        else:
            layers[current_layer].add_metadata_entry(digit)
            if layers[current_layer].num_metadata_entries_left == 0:
                current_layer -= 1

        if current_layer < 0:
            break

        if current_layer < len(layers) - 1:
            layers[current_layer].add_child_node(layers[-1])
            layers = layers[:-1]

            if layers[current_layer].num_child_nodes_left:
                current_layer += 1

        if current_layer >= len(layers):
            layers.append(Node.empty())

            is_header_state = True
            header_on_num_child_nodes = True

    # There's only one root node, so if this
    # throws, there's something wrong.
    assert len(layers) == 1

    return layers[0]


# The much, much, much, much, much, much more elegant
# solution by Micheal Marsalek.
def parse_digits(digit_list):
    # Essentially, this takes the insight I had and really distills it.
    # I initally knew this would need recursion, but I dismissed it,
    # thinking it would be impossible to tell when child nodes end
    # end and the metadata for the parent node begins. But in fact,
    # by using "no children" as a base case, you realize that a node
    # fundamentally only has a header and metadata. If you continue
    # going through headers if there are children, you will know for sure
    # that what you now have is not a header for a child node, but a metadata
    # value. So here, we...

    # ...take both the header values from beginning,
    num_children = digit_list.pop(0)
    num_metadata_entries = digit_list.pop(0)

    # ..go through each child and recurse on it. It'll pop off
    # its own header values and metadata, so on so forth...
    child_nodes = [parse_digits(digit_list) for _ in range(num_children)]

    # ... and because lists in Python are essentially pass-by-reference,
    # this results in *only* our own node's metadata values remaining
    # for us to pick up. If this node has a sibling, it won't overcollect
    # the sibling's header values as metadata because it's only concerned
    # with the scope of its children and metadata.
    metadata_entries = [digit_list.pop(0) for _ in range(num_metadata_entries)]

    return Node(child_nodes, metadata_entries)
