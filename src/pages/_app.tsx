import type {AppProps} from 'next/app'
import 'bootstrap/dist/css/bootstrap.min.css'
import './globals.scss'
import {Alert, ThemeProvider} from "react-bootstrap";
import React, {ReactNode, useContext, useState} from "react";
import {Variant} from "react-bootstrap/types";
import {Channel, invoke} from "@tauri-apps/api/core";

// ---- ALERT SYSTEM ----

const ALERT_TIMEOUT = 10

export interface LauncherAlert {
    variant: Variant,
    content: React.ReactNode,
    id: number
}

type AddAlert = (variant: Variant, content: ReactNode) => void

export const Alerts = React.createContext<AddAlert>(() => {});



// ---- CONSOLE LOG SYSTEM ----

export type ConsoleLine = {
    is_err: string,
    frag: number[]
}

type CCContext = {
    channel: Channel<ConsoleLine> | undefined,
    setChannel: (channel: Channel<ConsoleLine>) => void
};

export const ConsoleChannel = React.createContext<CCContext>({
    channel: undefined,
    setChannel: () => {}
})

export const useConsole = () => useContext(ConsoleChannel)

// ---- APP ENTRY ----

export default function MyApp({Component, pageProps}: AppProps) {
    const [alertCount, setAlertCount] = useState(0);
    const [alerts, setAlerts] = useState<LauncherAlert[]>([]);
    const [channel, setChannel] = useState<Channel<ConsoleLine> | undefined>(undefined)

    const addAlert: AddAlert = (variant: Variant, content: ReactNode) => {
        let id = alertCount + 1;
        const alert = {
            variant: variant,
            content: content,
            id: id
        }

        setAlertCount(id)

        alerts.push(alert)

        setTimeout(() => {
            setAlerts((newAlerts) => newAlerts.filter((t) => t.id != id))
        }, ALERT_TIMEOUT * 1000);
    }

    return <div data-bs-theme="dark">
        <ThemeProvider>
            <ConsoleChannel.Provider value={{
                channel,
                setChannel
            } as CCContext}>
                <Alerts.Provider value={addAlert}>
                    <Component {...pageProps} />
                    <div
                        style={{
                            position: "absolute",
                            bottom: 0,
                            right: 0,
                            zIndex: 20,
                            margin: "10px",
                            transition: "ease-in"
                        }}
                    >
                        {alerts.map((value, index) =>
                            <Alert key={index} variant={value.variant} dismissible>
                                {value.content}
                            </Alert>
                        )}
                    </div>
                </Alerts.Provider>
            </ConsoleChannel.Provider>
        </ThemeProvider>
    </div>
}

