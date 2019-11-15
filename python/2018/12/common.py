def render(state):
    print(''.join('^' if p else '`' for p in state.values()))
