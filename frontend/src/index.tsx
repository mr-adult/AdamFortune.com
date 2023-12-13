import { render } from 'preact';
import { LocationProvider, Router, Route } from 'preact-iso';

import { Home } from './pages/Home/index.jsx';
import { NotFound } from './pages/_404.jsx';
import './style.css';
import { Projects } from './pages/Projects/index.js';
import { NavBar } from './components/NavBar.js';

export function App() {
	return (
		<LocationProvider>
			<NavBar additional={[]} />
			<main>
				<Router>
					<Route path="/" component={Home} />
					<Route path="/projects" component={Projects} />
					<Route default component={NotFound} />
				</Router>
			</main>
		</LocationProvider>
	);
}

render(<App />, document.getElementById('app'));
