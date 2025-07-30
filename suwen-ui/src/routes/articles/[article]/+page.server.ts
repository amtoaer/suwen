export const load = async () => {
    const articleData = {
        title: `记一次对 Rust Embed 压缩的探索`,
        content: `<p>事情的起因是偶然发现 <a href="https://github.com/amtoaer/bili-sync">bili-sync</a> 的编译产物二进制过大，达到了惊人的 26M。由于我前段时间使用 <a href="https://crates.io/crates/rust-embed">Rust Embed</a>（以下简称 embed）将前端生成的静态文件打包到二进制中，自然怀疑是前端文件过大导致的。经过简单排查，发现自己编写的前端占用不到 1M，而嵌入的 Swagger-UI 却占用了 10+M。不过无论占比如何，这两者都是通过 embed 打包进二进制程序的，因此需要探索 embed 的压缩方案。</p>
<h2>embed 官方提供的压缩</h2>
<p>查看 embed 的官方 README，可以发现它提供了 <code>compression</code> feature flag：</p>
<blockquote>
<ul>
<li><code>compression</code>: Compress each file when embedding into the binary. Compression is done via <a href="https://crates.io/crates/include-flate">include-flate</a>.</li>
</ul>
</blockquote>
<p>那么问题到此解决了吗？并非如此。查看相关说明，embed 的 compression 是通过 include-flate 实现的，我们来看看 include-flate 的介绍：</p>
<blockquote>
<p>A variant of <code>include_bytes!</code>/<code>include_str!</code> with compile-time deflation and runtime lazy inflation.
一个带有编译时压缩与运行时懒解压的 <code>include_bytes!</code> / <code>include_str!</code> 变体。</p>
</blockquote>
<p>我们知道，像 embed 这类嵌入方案本质上都是对 <code>include_bytes!</code> / <code>include_str!</code> 的封装，这些宏在编译时展开后会将文件数据转换为静态数据结构包含在二进制文件中。include-flate 是这两者的变体，从介绍可以推测其运行原理：</p>
<ol>
<li>编译时将文件内容压缩，使用 <code>include_bytes!</code> 存储压缩后的数据；</li>
<li>运行时访问，如果文件已解压则直接返回，否则取出压缩数据，解压后缓存并返回。</li>
</ol>
<p>这种方案本质上是"透明压缩"，向外界提供与 <code>include_bytes!</code> 相同的 API，使用者无需感知内部的解压过程。但代价是内存浪费。</p>
<p>程序运行时既需要存储二进制中的压缩数据，又需要在文件首次访问时存储解压出的原始数据。对比未压缩方案：</p>
<ol>
<li>程序体积：原始数据 → 压缩后数据（减小）</li>
<li>内存占用：原始数据 → 原始数据 + 压缩后数据（增加）</li>
</ol>
<p>虽然我想压缩程序体积，但并不想浪费更多内存。要想既减小程序体积又缩小内存占用，需要寻找新的解决方案。</p>
<h2>Content-Encoding</h2>
<p>众所周知，HTTP 本身支持对响应体进行压缩，并提供 Content-Encoding 头来标识采用的压缩算法。考虑到使用 embed 的场景是托管静态文件，我们可以预先压缩静态文件，程序中仅打包压缩后的文件，当浏览器请求时直接返回压缩文件并设置相应的 Content-Encoding 头，让浏览器自行处理解压。这样就能两全其美，既缩小程序体积，又减少内存占用。</p>
<p>最初我想 fork 一份 embed 进行修改，但一堆裸露的 cfg! 判断与宏拼接让我萌生退意：
<img src="./attachments/Qma9efPHLAo9NT4ywTSm3s9bNpnfziyLkyBp3mab6gYopF?img-quality=75&amp;img-format=auto&amp;img-onerror=redirect&amp;img-width=3840" alt="embed" /></p>
<p>幸运的是，经过搜索我发现了一个与我需求类似的项目：<a href="https://github.com/SeriousBug/rust-embed-for-web">rust-embed-for-web</a>。该项目在 embed 基础上预先打包 gzip 和 br 两种压缩格式的文件，原始文件、gzip 压缩、br 压缩分别通过 <code>.data()</code>、<code>.data_gzip()</code>、<code>.data_br()</code> 访问，其中 <code>.data()</code> 必定存在，另外两个返回 <code>Option</code> 可通过宏参数控制。可以看出该项目同样会占用额外空间，只是采用预编译方式优化静态文件传输。要达到节约空间的目的，只需将 <code>.data()</code> 的返回值也改为 <code>Option</code>，并修改过程宏以支持是否存储源文件的配置。</p>
<p>这个项目的代码比 embed 清晰许多，修改起来非常方便。在修改过程中我还顺便用 <code>enum_dispatch</code> 将动态派发替换为静态派发，应该会带来一些性能提升，具体变更可查看<a href="https://github.com/amtoaer/rust-embed-for-web/commit/b6eeb475cbe1ad5cae02d5373a1bba12ea58a869">这个提交</a>。</p>
<hr />
<p>一点题外话：发现这个项目对 path =&gt; file 映射的实现是将宏展开为：</p>
<pre style="background-color:#2b303b;">
<span style="color:#b48ead;">match</span><span style="color:#c0c5ce;"> path {
</span><span style="color:#c0c5ce;">    path1 =&gt; file1,
</span><span style="color:#c0c5ce;">    path2 =&gt; file2,
</span><span style="color:#c0c5ce;">    ...
</span><span style="color:#c0c5ce;">}
</span></pre>
<p>而原始 embed 则展开为：</p>
<pre style="background-color:#2b303b;">
<span style="color:#b48ead;">const </span><span style="color:#d08770;">ENTRIES</span><span style="color:#c0c5ce;">: &amp;</span><span style="color:#b48ead;">&#39;static </span><span style="color:#c0c5ce;">[(&amp;</span><span style="color:#b48ead;">&#39;static str</span><span style="color:#c0c5ce;">, EmbeddedFile)] = [(path1, file1), (path2, file2), ...];
</span><span style="color:#b48ead;">let</span><span style="color:#c0c5ce;"> position = </span><span style="color:#d08770;">ENTRIES</span><span style="color:#c0c5ce;">.</span><span style="color:#96b5b4;">binary_search_by_key</span><span style="color:#c0c5ce;">(&amp;path.</span><span style="color:#96b5b4;">as_str</span><span style="color:#c0c5ce;">(), |</span><span style="color:#bf616a;">entry</span><span style="color:#c0c5ce;">| entry.</span><span style="color:#d08770;">0</span><span style="color:#c0c5ce;">);
</span><span style="color:#c0c5ce;">position.</span><span style="color:#96b5b4;">ok</span><span style="color:#c0c5ce;">().</span><span style="color:#96b5b4;">map</span><span style="color:#c0c5ce;">(|</span><span style="color:#bf616a;">idx</span><span style="color:#c0c5ce;">| </span><span style="color:#d08770;">ENTRIES</span><span style="color:#c0c5ce;">[idx].</span><span style="color:#d08770;">1</span><span style="color:#c0c5ce;">)
</span></pre>
<p>我不太了解 match 语句的具体匹配机制，不确定哪种方式性能更好，如果有大佬读到可以在评论中指点一二。</p>
<hr />
<h2>一个例外</h2>
<p>细心的朋友应该注意到，我上面的改动除了支持"是否存储源文件"开关外，还添加了一个"except"条件。这是因为 Swagger-UI 有一个特殊文件需要服务端动态替换：</p>
<pre style="background-color:#2b303b;">
<span style="color:#b48ead;">pub fn </span><span style="color:#8fa1b3;">serve</span><span style="color:#c0c5ce;">&lt;</span><span style="color:#b48ead;">&#39;a</span><span style="color:#c0c5ce;">&gt;(
</span><span style="color:#c0c5ce;">    </span><span style="color:#bf616a;">path</span><span style="color:#c0c5ce;">: &amp;</span><span style="color:#b48ead;">str</span><span style="color:#c0c5ce;">,
</span><span style="color:#c0c5ce;">    </span><span style="color:#bf616a;">config</span><span style="color:#c0c5ce;">: Arc&lt;Config&lt;</span><span style="color:#b48ead;">&#39;a</span><span style="color:#c0c5ce;">&gt;&gt;,
</span><span style="color:#c0c5ce;">) -&gt; Result&lt;Option&lt;SwaggerFile&lt;</span><span style="color:#b48ead;">&#39;a</span><span style="color:#c0c5ce;">&gt;&gt;, Box&lt;dyn Error&gt;&gt; {
</span><span style="color:#c0c5ce;">    </span><span style="color:#b48ead;">let mut</span><span style="color:#c0c5ce;"> file_path = path;
</span><span style="color:#c0c5ce;">
</span><span style="color:#c0c5ce;">    </span><span style="color:#b48ead;">if</span><span style="color:#c0c5ce;"> file_path.</span><span style="color:#96b5b4;">is_empty</span><span style="color:#c0c5ce;">() || file_path == &quot;</span><span style="color:#a3be8c;">/</span><span style="color:#c0c5ce;">&quot; {
</span><span style="color:#c0c5ce;">        file_path = &quot;</span><span style="color:#a3be8c;">index.html</span><span style="color:#c0c5ce;">&quot;;
</span><span style="color:#c0c5ce;">    }
</span><span style="color:#c0c5ce;">
</span><span style="color:#c0c5ce;">    </span><span style="color:#b48ead;">if let </span><span style="color:#c0c5ce;">Some(file) = SwaggerUiDist::get(file_path) {
</span><span style="color:#c0c5ce;">        </span><span style="color:#b48ead;">let mut</span><span style="color:#c0c5ce;"> bytes = file.data;
</span><span style="color:#c0c5ce;">        </span><span style="color:#65737e;">// 罪魁祸首 &quot;swagger-initializer.js&quot;，可恶啊！
</span><span style="color:#c0c5ce;">        </span><span style="color:#b48ead;">if</span><span style="color:#c0c5ce;"> file_path == &quot;</span><span style="color:#a3be8c;">swagger-initializer.js</span><span style="color:#c0c5ce;">&quot; {
</span><span style="color:#c0c5ce;">            </span><span style="color:#b48ead;">let mut</span><span style="color:#c0c5ce;"> file = </span><span style="color:#b48ead;">match </span><span style="color:#c0c5ce;">String::from_utf8(bytes.</span><span style="color:#96b5b4;">to_vec</span><span style="color:#c0c5ce;">()) {
</span><span style="color:#c0c5ce;">                Ok(file) =&gt; file,
</span><span style="color:#c0c5ce;">                Err(error) =&gt; </span><span style="color:#b48ead;">return </span><span style="color:#c0c5ce;">Err(Box::new(error)),
</span><span style="color:#c0c5ce;">            };
</span><span style="color:#c0c5ce;">
</span><span style="color:#c0c5ce;">            file = </span><span style="color:#96b5b4;">format_config</span><span style="color:#c0c5ce;">(config.</span><span style="color:#96b5b4;">as_ref</span><span style="color:#c0c5ce;">(), file)?;
</span><span style="color:#c0c5ce;">
</span><span style="color:#c0c5ce;">            </span><span style="color:#b48ead;">if let </span><span style="color:#c0c5ce;">Some(oauth) = &amp;config.oauth {
</span><span style="color:#c0c5ce;">                </span><span style="color:#b48ead;">match </span><span style="color:#c0c5ce;">oauth::format_swagger_config(oauth, file) {
</span><span style="color:#c0c5ce;">                    Ok(oauth_file) =&gt; file = oauth_file,
</span><span style="color:#c0c5ce;">                    Err(error) =&gt; </span><span style="color:#b48ead;">return </span><span style="color:#c0c5ce;">Err(Box::new(error)),
</span><span style="color:#c0c5ce;">                }
</span><span style="color:#c0c5ce;">            }
</span><span style="color:#c0c5ce;">
</span><span style="color:#c0c5ce;">            bytes = Cow::Owned(file.</span><span style="color:#96b5b4;">as_bytes</span><span style="color:#c0c5ce;">().</span><span style="color:#96b5b4;">to_vec</span><span style="color:#c0c5ce;">())
</span><span style="color:#c0c5ce;">        };
</span><span style="color:#c0c5ce;">
</span><span style="color:#c0c5ce;">        Ok(Some(SwaggerFile {
</span><span style="color:#c0c5ce;">            bytes,
</span><span style="color:#c0c5ce;">            content_type: mime_guess::from_path(file_path)
</span><span style="color:#c0c5ce;">                .</span><span style="color:#96b5b4;">first_or_octet_stream</span><span style="color:#c0c5ce;">()
</span><span style="color:#c0c5ce;">                .</span><span style="color:#96b5b4;">to_string</span><span style="color:#c0c5ce;">(),
</span><span style="color:#c0c5ce;">        }))
</span><span style="color:#c0c5ce;">    } </span><span style="color:#b48ead;">else </span><span style="color:#c0c5ce;">{
</span><span style="color:#c0c5ce;">        Ok(None)
</span><span style="color:#c0c5ce;">    }
</span><span style="color:#c0c5ce;">}
</span></pre>
<p>由于 swagger-initializer.js 需要服务端实时替换模板内容，必须保留原始文件。为了兼容这个特殊文件，只能添加一下 <code>preserve_source_except</code> 处理。</p>
<h2>合并回 bili-sync</h2>
<p>完成上述工作后，就可以将改动合并回 <code>bili-sync</code> 了。首先修改 Asset 的参数：</p>
<pre style="background-color:#2b303b;">
<span style="color:#c0c5ce;">#[</span><span style="color:#bf616a;">derive</span><span style="color:#c0c5ce;">(RustEmbed)]
</span><span style="color:#c0c5ce;">#[</span><span style="color:#bf616a;">preserve_source </span><span style="color:#c0c5ce;">= </span><span style="color:#bf616a;">false</span><span style="color:#c0c5ce;">] </span><span style="color:#65737e;">// 不保留原始文件
</span><span style="color:#c0c5ce;">#[</span><span style="color:#bf616a;">gzip </span><span style="color:#c0c5ce;">= </span><span style="color:#bf616a;">false</span><span style="color:#c0c5ce;">] </span><span style="color:#65737e;">// 不开启 gzip（仅开启 br）
</span><span style="color:#c0c5ce;">#[</span><span style="color:#bf616a;">folder </span><span style="color:#c0c5ce;">= &quot;</span><span style="color:#a3be8c;">../../web/build</span><span style="color:#c0c5ce;">&quot;]
</span><span style="color:#b48ead;">struct </span><span style="color:#c0c5ce;">Asset;
</span></pre>
<p>然后在静态资源访问部分添加 <code>Content-Encoding</code> 头：</p>
<pre style="background-color:#2b303b;">
<span style="color:#b48ead;">let </span><span style="color:#c0c5ce;">Some(content) = Asset::get(path) </span><span style="color:#b48ead;">else </span><span style="color:#c0c5ce;">{
</span><span style="color:#c0c5ce;">    </span><span style="color:#b48ead;">return </span><span style="color:#c0c5ce;">(StatusCode::</span><span style="color:#d08770;">NOT_FOUND</span><span style="color:#c0c5ce;">, &quot;</span><span style="color:#a3be8c;">404 Not Found</span><span style="color:#c0c5ce;">&quot;).</span><span style="color:#96b5b4;">into_response</span><span style="color:#c0c5ce;">();
</span><span style="color:#c0c5ce;">};
</span><span style="color:#c0c5ce;">Response::builder()
</span><span style="color:#c0c5ce;">    .</span><span style="color:#96b5b4;">status</span><span style="color:#c0c5ce;">(StatusCode::</span><span style="color:#d08770;">OK</span><span style="color:#c0c5ce;">)
</span><span style="color:#c0c5ce;">    .</span><span style="color:#96b5b4;">header</span><span style="color:#c0c5ce;">(
</span><span style="color:#c0c5ce;">        header::</span><span style="color:#d08770;">CONTENT_TYPE</span><span style="color:#c0c5ce;">,
</span><span style="color:#c0c5ce;">        content.</span><span style="color:#96b5b4;">mime_type</span><span style="color:#c0c5ce;">().</span><span style="color:#96b5b4;">as_deref</span><span style="color:#c0c5ce;">().</span><span style="color:#96b5b4;">unwrap_or</span><span style="color:#c0c5ce;">(&quot;</span><span style="color:#a3be8c;">application/octet-stream</span><span style="color:#c0c5ce;">&quot;),
</span><span style="color:#c0c5ce;">    )
</span><span style="color:#c0c5ce;">    .</span><span style="color:#96b5b4;">header</span><span style="color:#c0c5ce;">(header::</span><span style="color:#d08770;">CONTENT_ENCODING</span><span style="color:#c0c5ce;">, &quot;</span><span style="color:#a3be8c;">br</span><span style="color:#c0c5ce;">&quot;)
</span><span style="color:#c0c5ce;">    </span><span style="color:#65737e;">// safety: \`RustEmbed\` will always generate br-compressed files if the feature is enabled
</span><span style="color:#c0c5ce;">    .</span><span style="color:#96b5b4;">body</span><span style="color:#c0c5ce;">(Body::from(content.</span><span style="color:#96b5b4;">data_br</span><span style="color:#c0c5ce;">().</span><span style="color:#96b5b4;">unwrap</span><span style="color:#c0c5ce;">()))
</span><span style="color:#c0c5ce;">    .</span><span style="color:#96b5b4;">unwrap_or_else</span><span style="color:#c0c5ce;">(|_| {
</span><span style="color:#c0c5ce;">        </span><span style="color:#b48ead;">return </span><span style="color:#c0c5ce;">(StatusCode::</span><span style="color:#d08770;">INTERNAL_SERVER_ERROR</span><span style="color:#c0c5ce;">, &quot;</span><span style="color:#a3be8c;">500 Internal Server Error</span><span style="color:#c0c5ce;">&quot;).</span><span style="color:#96b5b4;">into_response</span><span style="color:#c0c5ce;">();
</span><span style="color:#c0c5ce;">    })
</span></pre>
<p>经过多次编译测试，结果如下：</p>
<table><thead><tr><th></th><th>不包含前端</th><th>包含 Swagger-UI</th><th>包含 Swagger-UI + 前端</th></tr></thead><tbody>
<tr><td>压缩前</td><td>13M</td><td>25M</td><td>26M</td></tr>
<tr><td>压缩后</td><td>13M</td><td>-</td><td>16M</td></tr>
</tbody></table>
<p>效果立竿见影，压缩效果显著。</p>
`,
        summary: `事情的起因是发现 bili-sync 的编译产物二进制过大，达到 26M。经过排查，发现前端文件占用不到 1M，而嵌入的 Swagger-UI 却占用了 10+M。为了减小程序体积，需要探索 Rust Embed（embed）的压缩方案。embed 提供了 compression 特性，通过 include-flate 实现文件的编译时压缩和运行时解压，但这种方案会增加内存占用。为了解决这个问题，考虑使用 HTTP 的 Content-Encoding 头来预先压缩静态文件，程序中只打包压缩后的文件，浏览器请求时直接返回压缩文件。经过搜索，发现 rust-embed-for-web 项目可以满足需求，该项目支持 gzip 和 br 两种压缩格式，并且代码清晰，修改方便。在修改过程中，还优化了动态派发为静态派发。由于 Swagger-UI 有一个特殊文件需要动态替换，必须保留原始文件，因此添加了 preserve_source_except 处理。最终将改动合并回 bili-sync，并在静态资源访问部分添加 Content-Encoding 头。经过多次编译测试，结果显示程序体积得到了有效缩减。`,
        tags: ['Rust', 'Svelte'],
        views: 123,
        publishedDate: new Date('2023-10-01'),
        comments: 45,
    };
    return {
        articleData,
    };
}