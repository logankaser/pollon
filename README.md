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
<article>
<h3>Node</h3>
Each node is just an HTML fragment stored on the server as a file
For example a document with one node would look like:
`document/0.html`
<pre>html
&ltarticle>
Unformatted text
&lt/article>
</pre>
As nodes are normal HTML files they can be edited by hand or read by basic tools.
You could swap the order of nodes with just `mv` .
Render a document from within the document folder:
<pre>cat $(ls -1v)</pre> 
Render and open in browser:
<pre>d=/tmp/doc.html;cat $(ls -1v) > $d;xdg-open $d</pre>
</article>
<article>
<h2>API</h2>
The document is exposed to clients with HTTP:
<pre>
GET /{document}
GET /{document}?nodes=1,2,3,8,9
POST /{document} // append
GET /{document}/{node}
PUT /{document}/{node} // replace
DELETE /{document}/{node} // remove
</pre>
</article>
<article>
<h2>Client</h2>
Clients connect to a Pollon node and provide richer editing and viewing.
<h4>Examples</h4>
<lt>
<li>Render documents
<li>Allow addition, editing, deletion, and reordering of node(s)
<li>Friendly editing of HTML data, with raw text fallback for unsupported tags
<li>Provide styling / CSS
</lt>
</article>
<article>
<h2>Philosophy</h2>
<lt>
<li>Approachable. If you can open and print files, you can script pollon
<li>Simple. <em>really</em> simple
<li>Don't fight the metaphore of the platform: Embrace files. Embrace URLs/HTML
<li>Don't make a 700mb electron app with whiteboarding and 4 kinds of tagging
</lt>
</article>
