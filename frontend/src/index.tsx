import { render } from 'preact';
import { LocationProvider, Router, Route } from 'preact-iso';

import { Home } from './pages/Home/index.jsx';
import { NotFound } from './pages/_404.jsx';
import { Projects } from './pages/Projects/index.js';
import { NavBar } from './components/NavBar.js';
import { Blog } from './pages/Blog/index.js';
import './style.css';
import { BlogPost } from './components/BlogPost.js';
import { Repo } from './pages/Repo/index.js';

export function App() {
	return (
		<LocationProvider>
			<main>
				<Router>
					<Route path="/" component={Home} />
					<Route path="/projects" component={Projects} />
					<Route path="/projects/:project" component={Repo as any} />
					<Route path="/blog" component={Blog as any} />
					<Route path="/blog/:blogpost" component={BlogPost as any} />
					<Route default component={NotFound} />
				</Router>
			</main>
		</LocationProvider>
	);
}

render(<App />, document.getElementById('app'));
