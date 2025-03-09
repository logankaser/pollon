<article>
<h2>Document</h2>
A document is a collection of nodes
<pre>
document/
|-0.html
|-1.html
|-2.html
...
</pre>
A document is normal folder.
The only files it contains are it’s nodes
</article>
<article contenteditable="true">
<h3>Node</h3>
Each node is just an HTML fragment stored on the server as a file
For example a document with one node would look like:
`document/0.html`
<pre>html
&lt;article&gt;
Unformatted text
&lt;/article&gt;
</pre>
As nodes are normal HTML files they can be edited by hand or read by basic tools.
You could swap the order of nodes with just `mv` .
Render a document from within the document folder:
<pre>cat $(ls -1v *.html)</pre>
Render and open in browser:
<pre>d=/tmp/doc.html;cat $(ls -1v *.html) &gt; $d;xdg-open $d</pre></article>
<article contenteditable="true">
<h2>API</h2>
The document is exposed to clients with HTTP:
<pre>GET /{document}
GET /{document}/{node}
PUT /{document}/{node}
DELETE /{document}/{node}</pre>
</article><article contenteditable="true">
<h2>Client</h2>
Clients connect to a Pollon node and provide richer editing and viewing.
<p>Examples:</p>
<lt>
<li>Allow addition, editing, deletion, and reordering of node(s)
<li>Friendly editing of nodes
<li>Provide styling / CSS management
<li>Real-time sync
</lt>
</article>
<article>
<h2>Philosophy</h2>
<lt>
<li>Pursue the ideal of hypertext
<li>Don't fight the the platform, use files, URLs, and HTML
<li>Don't make a 700mb electron app with whiteboarding and 4 kinds of tagging
<li>Don't sacrifice time on the altar of JS build systems
<li>Don't make a plugin system
</lt>
</article>
