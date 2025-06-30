// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="md/引言.html">引言</a></li><li class="chapter-item expanded "><a href="md/线程模型.html"><strong aria-hidden="true">1.</strong> 线程模型</a></li><li class="chapter-item expanded "><a href="md/内核架构.html"><strong aria-hidden="true">2.</strong> 内核架构</a></li><li class="chapter-item expanded "><a href="md/内存管理.html"><strong aria-hidden="true">3.</strong> 内存管理</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="md/内存分配器.html"><strong aria-hidden="true">3.1.</strong> 内存分配器</a></li><li class="chapter-item expanded "><a href="md/页表.html"><strong aria-hidden="true">3.2.</strong> 页表</a></li><li class="chapter-item expanded "><a href="md/地址空间.html"><strong aria-hidden="true">3.3.</strong> 地址空间</a></li></ol></li><li class="chapter-item expanded "><a href="md/任务管理.html"><strong aria-hidden="true">4.</strong> 任务管理</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="md/进程管理.html"><strong aria-hidden="true">4.1.</strong> 进程管理</a></li><li class="chapter-item expanded "><a href="md/用户线程管理.html"><strong aria-hidden="true">4.2.</strong> 用户线程管理</a></li><li class="chapter-item expanded "><a href="md/内核线程管理.html"><strong aria-hidden="true">4.3.</strong> 内核线程管理</a></li><li class="chapter-item expanded "><a href="md/任务调度.html"><strong aria-hidden="true">4.4.</strong> 任务调度</a></li></ol></li><li class="chapter-item expanded "><a href="md/异步管理.html"><strong aria-hidden="true">5.</strong> 异步管理</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="md/异步协程.html"><strong aria-hidden="true">5.1.</strong> 异步协程</a></li><li class="chapter-item expanded "><a href="md/协程执行器.html"><strong aria-hidden="true">5.2.</strong> 协程执行器</a></li><li class="chapter-item expanded "><a href="md/异步系统调用.html"><strong aria-hidden="true">5.3.</strong> 异步系统调用</a></li></ol></li><li class="chapter-item expanded "><a href="md/内核线程服务模型.html"><strong aria-hidden="true">6.</strong> 内核线程服务模型</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="md/自定义请求类型.html"><strong aria-hidden="true">6.1.</strong> 自定义请求类型</a></li><li class="chapter-item expanded "><a href="md/请求处理器.html"><strong aria-hidden="true">6.2.</strong> 请求处理器</a></li><li class="chapter-item expanded "><a href="md/文件系统服务线程.html"><strong aria-hidden="true">6.3.</strong> 文件系统服务线程</a></li><li class="chapter-item expanded "><a href="md/内核线程故障恢复.html"><strong aria-hidden="true">6.4.</strong> 内核线程故障恢复</a></li></ol></li><li class="chapter-item expanded "><a href="md/文件系统.html"><strong aria-hidden="true">7.</strong> 文件系统</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="md/内核fs.html"><strong aria-hidden="true">7.1.</strong> 内核使用easy-fs</a></li><li class="chapter-item expanded "><a href="md/文件类型.html"><strong aria-hidden="true">7.2.</strong> 文件类型</a></li></ol></li><li class="chapter-item expanded "><a href="md/中断处理和系统调用.html"><strong aria-hidden="true">8.</strong> 中断处理和系统调用</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="md/中断处理.html"><strong aria-hidden="true">8.1.</strong> 中断处理</a></li><li class="chapter-item expanded "><a href="md/系统调用.html"><strong aria-hidden="true">8.2.</strong> 系统调用</a></li></ol></li><li class="chapter-item expanded "><a href="md/总结与计划.html"><strong aria-hidden="true">9.</strong> 总结与计划</a></li><li class="chapter-item expanded "><a href="md/效果演示.html"><strong aria-hidden="true">10.</strong> 附录：效果演示</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
