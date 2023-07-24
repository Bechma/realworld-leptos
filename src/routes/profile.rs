use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[server(EditorAction, "/api")]
#[tracing::instrument]
pub async fn editor_action(cx: Scope) -> Result<(), ServerFnError> {
    Ok(())
}

#[tracing::instrument]
#[component]
pub fn Profile(cx: Scope) -> impl IntoView {
    let error = create_rw_signal(cx, view! {cx, <ul></ul>});

    let editor_server_action = create_server_action::<EditorAction>(cx);
    let result_of_call = editor_server_action.value();

    let params = use_params_map(cx);
    let user = params.get().get("user").cloned().unwrap_or_default();
    let navigate = use_navigate(cx);
    let profile_title = format!("{user}'s profile");

    view! { cx,
        <Title text=profile_title/>
        <div class="profile-page">
            <div class="user-info">
                <div class="container">
                    <div class="row">
                        <div class="col-xs-12 col-md-10 offset-md-1">
                            <img src="{{user.image}}" class="user-img" />
                            <h4>{{user.username}}</h4>
                            <p>{% if user.bio %}{{user.bio}}{% else %}No bio available{% endif %}</p>
                            {{buttons::follow(user=user.username, following=user.following)}}
                        </div>
                    </div>
                </div>
            </div>

            <div class="container">
                <div class="row">
                    <div class="col-xs-12 col-md-10 offset-md-1">
                        <div class="articles-toggle">
                            <ul class="nav nav-pills outline-active">
                                <li class="nav-item">
                                    <a class="nav-link {% if not favourites %}active{% endif %}"
                                        href="{{current}}">{{user.username}}'s Articles</a>
                                </li>
                                <li class="nav-item">
                                    <a class="nav-link {% if favourites %}active{% endif %}"
                                        href="{{current}}?favourites=true">Favorited Articles</a>
                                </li>
                            </ul>
                        </div>

                        {% for a in articles %}
                        <div class="article-preview">
                            {{macros::preview(article=a)}}
                        </div>
                        {% endfor %}
                    </div>
                </div>
            </div>
        </div>
    }
}
