use leptos::*;
use leptos_meta::*;

const PRISM_SCRIPT: &str = include_str!("../prism.js");
const GENERAL_STYLE: &str = include_str!("../styles.css");
const PRISM_STYLE: &str = include_str!("../prism.css");

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let the_code = RwSignal::new(
        r#"
          fn main() {
            println!("hello world");
          }"#,
    );
    view! {
    <>
    <Style>
    {
       String::from("") +
       PRISM_STYLE +
       GENERAL_STYLE
    }
    </Style>
    <pre class="fullpage">
      <code class="language-rust line-numbers">
      {
        the_code.get()
      }
      </code>
    </pre>
    <script>
    {
      PRISM_SCRIPT
    }
    </script>
    </>

      }
}
