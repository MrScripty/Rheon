"""Package source-context Markdown links and images without invoking numerical code."""
from pathlib import Path
from urllib.parse import urlsplit, unquote
import json, os, zipfile


def write_bundle(repo, output, front, reading, files, command):
    repo, output = Path(repo), Path(output)
    book = repo/'docs/research-book'
    entries = {}

    def parse(text):
        return json.loads(command(['pandoc', '-f', 'markdown+tex_math_single_backslash', '-t', 'json'], input=text))

    def render(ast):
        return command(['pandoc', '-f', 'json', '-t', 'markdown+tex_math_single_backslash', '--wrap=none'], input=json.dumps(ast)).encode()

    def destination(source):
        if source.parent in (book/'chapters', book/'appendices'):
            return 'chapters/'+source.name
        if source.parent == book/'implementation':
            return 'implementation/'+source.name
        if source.parent == repo/'proofs/Rheon':
            return 'proofs/'+source.name
        return 'source-files/'+str(source.relative_to(repo))

    def package(source):
        target = destination(source)
        if target not in entries:
            entries[target] = b''  # Admit cyclic links before traversing them.
            entries[target] = render(rewrite(parse(source.read_text()), source, target)) if source.suffix == '.md' else source.read_bytes()
        return target

    def rewrite(ast, source, name):
        def walk(node):
            if isinstance(node, dict):
                if node.get('t') in ('Link', 'Image'):
                    target = node['c'][-1][0]
                    url = urlsplit(target)
                    if not url.scheme and not url.netloc and url.path:
                        local = (source.parent/unquote(url.path)).resolve()
                        if local == repo/'docs/education/native-labs.html':
                            dest = 'README.md'
                            fragment = 'native-playback'
                        elif local == repo/'docs/education/aligned-strain-lab.html':
                            dest = 'README.md'
                            fragment = 'interactive-aligned-strain'
                        elif local == repo/'docs/education/obstacle-flow-lab.html':
                            dest = 'README.md'
                            fragment = 'obstacle-flow'
                        elif local == repo/'docs/education/obstacle-lab.html':
                            dest = 'README.md'
                            fragment = 'static-geometry'
                        elif url.path.startswith('figures/') and (output/url.path).is_file():
                            dest = url.path
                            entries[dest] = (output/dest).read_bytes()
                            fragment = url.fragment
                        else:
                            if not local.is_relative_to(repo) or not local.is_file():
                                raise ValueError('Unpackaged Markdown target: '+str(source)+': '+target)
                            dest = package(local)
                            fragment = url.fragment
                        node['c'][-1][0] = os.path.relpath(dest, str(Path(name).parent))+('#'+fragment if fragment else '')
                for value in node.values():walk(value)
            elif isinstance(node, list):
                for value in node:walk(value)
        walk(ast)
        return ast

    combined = parse(front)
    for path, text in reading:
        combined['blocks'].extend(rewrite(parse(text), path, 'Rheon-expanded-book.md')['blocks'])
    entries['Rheon-expanded-book.md'] = render(combined)
    for path in files:package(path.resolve())
    for path in sorted((output/'figures').rglob('*')):
        if path.is_file():entries[str(path.relative_to(output))] = path.read_bytes()
    entries['README.md'] = b'''# Rheon Markdown companion

Extract this ZIP with all relative paths intact. The reading manuscript,
individual guides, linked evidence, source and illustrations are packaged.

## Native playback

The three native models are recorded playback, described in the
[native progression](implementation/native-wall-force-sequence.md).
This Markdown companion contains documentation, not a browser laboratory.
Open native-labs.html in the separately built static HTML edition for the hub.
A source-only HTML build explicitly states that playback bundles are absent.
No example, integration or reference computation was run to make this archive.

## Static geometry

The [static obstacle guide](implementation/static-obstacle-geometry.md) describes
the bounded owner and native controls. The separately built HTML edition provides
obstacle-lab.html when qualified records are supplied; its source-only build
states absence. This Markdown archive contains no native computation or lab.

## Obstacle flow

The [bounded pressure and reduced-shear guide](implementation/static-obstacle-flow.md)
explains the separately scoped field responses and exact rational controls.
The HTML edition supplies obstacle-flow-lab.html when qualified native records
are supplied. This archive does not execute either operator.

## Interactive aligned strain

The [finite strain guide](implementation/aligned-strain-laboratory.md) explains
the retained native specimen, row work and exact-real theorem premises. Open
aligned-strain-lab.html in the separately built HTML edition for interactive
finite algebra when a qualified packet is included. This Markdown companion
contains documentation and source, not an executing browser lab.
'''
    (output/'downloads/Rheon-expanded-book.md').write_bytes(entries['Rheon-expanded-book.md'])
    with zipfile.ZipFile(output/'downloads/Rheon-expanded-markdown.zip', 'w', zipfile.ZIP_DEFLATED) as bundle:
        for name, data in sorted(entries.items()):bundle.writestr(name, data)
