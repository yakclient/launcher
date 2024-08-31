import type {AppProps} from 'next/app'
import 'bootstrap/dist/css/bootstrap.min.css'
import './globals.scss'
import {ThemeProvider} from "react-bootstrap";

export default function MyApp({Component, pageProps}: AppProps) {
    return <>
        <ThemeProvider>
            <Component {...pageProps} />
        </ThemeProvider>
    </>
}