// MuJS Worker script for mutool run
// Communicates via line-delimited JSON over stdin/stdout.

var currentDoc = null;
var currentPath = null;

function getDoc(path) {
    if (currentDoc && currentPath === path) {
        return currentDoc;
    }
    currentDoc = null;
    gc();
    currentDoc = Document.openDocument(path);
    currentPath = path;
    return currentDoc;
}

function closeDoc() {
    currentDoc = null;
    currentPath = null;
    gc();
}
function extractStructuredTextWithWords(page) {
    var st = page.toStructuredText("preserve-whitespace,preserve-ligatures");
    var docJson = JSON.parse(st.asJSON(1));

    var linesWords = [];
    var currentLineWords = [];
    var currentWordChars = [];
    var currentWordMinX = 0, currentWordMinY = 0, currentWordMaxX = 0, currentWordMaxY = 0;

    function flushWord() {
        if (currentWordChars.length > 0) {
            currentLineWords.push({
                text: currentWordChars.join(""),
                bbox: {
                    x: Math.round(currentWordMinX * 10) / 10,
                    y: Math.round(currentWordMinY * 10) / 10,
                    w: Math.round((currentWordMaxX - currentWordMinX) * 10) / 10,
                    h: Math.round((currentWordMaxY - currentWordMinY) * 10) / 10
                }
            });
            currentWordChars = [];
        }
    }

    st.walk({
        beginLine: function() {
            currentLineWords = [];
            currentWordChars = [];
        },
        onChar: function(c, origin, font, size, quad) {
            if (c === " " || c === "\t" || c === "\n" || c === "\r") {
                flushWord();
                return;
            }

            var minX = Math.min(quad[0], quad[2], quad[4], quad[6]);
            var maxX = Math.max(quad[0], quad[2], quad[4], quad[6]);
            var minY = Math.min(quad[1], quad[3], quad[5], quad[7]);
            var maxY = Math.max(quad[1], quad[3], quad[5], quad[7]);

            if (currentWordChars.length === 0) {
                currentWordMinX = minX;
                currentWordMinY = minY;
                currentWordMaxX = maxX;
                currentWordMaxY = maxY;
            } else {
                if (minX < currentWordMinX) currentWordMinX = minX;
                if (minY < currentWordMinY) currentWordMinY = minY;
                if (maxX > currentWordMaxX) currentWordMaxX = maxX;
                if (maxY > currentWordMaxY) currentWordMaxY = maxY;
            }
            currentWordChars.push(c);
        },
        endLine: function() {
            flushWord();
            linesWords.push(currentLineWords);
        }
    });

    var lineIdx = 0;
    if (docJson.blocks) {
        for (var b = 0; b < docJson.blocks.length; b++) {
            var block = docJson.blocks[b];
            if (block.type === "text" && block.lines) {
                for (var l = 0; l < block.lines.length; l++) {
                    var line = block.lines[l];
                    line.words = linesWords[lineIdx] || [];
                    lineIdx++;
                }
            }
        }
    }

    return docJson;
}

function handleRequest(req) {
    var cmd = req.cmd;
    if (cmd === "ping") {
        return "pong";
    }
    if (cmd === "open") {
        var doc = getDoc(req.path);
        return {
            page_count: doc.countPages()
        };
    }
    if (cmd === "close") {
        closeDoc();
        return true;
    }
    if (cmd === "get_page_info") {
        var doc = getDoc(req.path);
        var total = doc.countPages();
        var pages = [];
        for (var i = 0; i < total; i++) {
            var page = doc.loadPage(i);
            var bounds = page.getBounds();
            var w = Math.abs(bounds[2] - bounds[0]);
            var h = Math.abs(bounds[3] - bounds[1]);
            pages.push({ width: w, height: h });
            page = null;
            if (i > 0 && i % 100 === 0) {
                gc();
            }
        }
        gc();
        return { total_pages: total, pages: pages };
    }
    if (cmd === "get_page_size") {
        var doc = getDoc(req.path);
        var pageIndex = (req.page || 1) - 1;
        var page = doc.loadPage(pageIndex);
        var bounds = page.getBounds();
        var w = Math.abs(bounds[2] - bounds[0]);
        var h = Math.abs(bounds[3] - bounds[1]);
        page = null;
        return { width: w, height: h };
    }
    if (cmd === "get_page_links") {
        var doc = getDoc(req.path);
        var pageIndex = (req.page || 1) - 1;
        var page = doc.loadPage(pageIndex);
        var rawLinks = page.getLinks() || [];
        var result = [];
        for (var i = 0; i < rawLinks.length; i++) {
            var l = rawLinks[i];
            var b = l.getBounds();
            var x0 = b[0];
            var y0 = b[1];
            var x1 = b[2];
            var y1 = b[3];
            var uri = l.getURI() || "";
            var isExt = l.isExternal();
            var destPage = null;
            if (!isExt && uri) {
                var m = uri.match(/#page=(\d+)/);
                if (m && m[1]) {
                    destPage = parseInt(m[1], 10);
                }
            }
            result.push({
                bbox: {
                    x: x0,
                    y: y0,
                    w: Math.abs(x1 - x0),
                    h: Math.abs(y1 - y0)
                },
                uri: uri,
                dest_page: destPage
            });
        }
        page = null;
        return result;
    }
    if (cmd === "get_stext") {
        var doc = getDoc(req.path);
        var pageIndex = (req.page || 1) - 1;
        var page = doc.loadPage(pageIndex);
        var result = extractStructuredTextWithWords(page);
        page = null;
        return result;
    }
    throw new Error("Unknown command: " + cmd);
}

while (true) {
    var line;
    try {
        line = readline();
    } catch (e) {
        // EOF reached
        break;
    }
    if (!line || line.trim() === "") continue;
    var req;
    try {
        req = JSON.parse(line);
    } catch (e) {
        print(JSON.stringify({ id: null, error: "Invalid JSON: " + String(e) }));
        continue;
    }
    try {
        var res = handleRequest(req);
        if (res && res.__raw_json) {
            print('{"id":' + JSON.stringify(req.id) + ',"result":' + res.__raw_json + '}');
        } else {
            print(JSON.stringify({ id: req.id, result: res }));
        }
    } catch (err) {
        print(JSON.stringify({ id: req.id, error: String(err) }));
    }
}
