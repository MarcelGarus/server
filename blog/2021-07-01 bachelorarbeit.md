# My Bachelor's Thesis

## Architektur und Implementierung eines modularen, skalierbaren Backends für Sensordaten verschiedener Nutzer

Meine Bachelorarbeit gibt's [hier zum runterladen](files/Bachelorarbeit.pdf). Eine Zusammenfassung:

> Sensoren in Zügen oder in deren Frachtgut können für mehr Kundenkomfort, eine zielgerichtete Instandhaltung und bessere Planbarkeit sorgen.
> Eine Sensorplattform sollte wiederverwendbar sein und einen „Live“-Datenzugriff ermöglichen.
> Vorhandene Frameworks wie AWS IoT Greengrass, Google Cloud IoT Core, Microsoft Azure RTOS oder Eclipse IoT binden die Plattform an eine spezifische Cloud oder lösen nur kleine Teilprobleme.
> In unserem Bachelorprojekt entwickeln wir deshalb eine eigene Plattform.
> In Wagen kann dazu Infrastruktur eingebaut werden, zu der sich Sensoren drahtlos verbinden können.
> Ein zentraler Server übernimmt die Verwaltung von Sensoren.
> Die Zuginfrastruktur sendet Messwerte direkt an Server, die von den Nutzern mithilfe einer bereitgestellten Bibliothek implementiert und selbst betrieben werden.
> Vorteil dieser Architektur gegenüber anderen betrachteten ist vor allem eine Unabhängigkeit der Nutzer vom Plattformbetreiber bezüglich Datenspeicherung und -verarbeitung.
> Die Implementierung der Backend-Komponenten verwendet Rust, asynchrone Programmierung, das Kommunikationsformat Cap’n Proto und die Datenbank MongoDB.
> Abhängig von der Anzahl der Verbindungen und versendeten Messwerte wird die CPU-, RAM- und Netzwerkauslastung der bereitgestellten Bibliothek betrachtet.
> Die Netzwerklast lässt sich durch Datenkompression während der Übertragung reduzieren; dabei werden ein Cap’n-Proto-spezifisches Packing sowie die Kompressionsverfahren zlib und LZ4 verglichen.
> Das Ausführen von Code direkt in Zügen kann mit einer Vorverarbeitung und Filterung der Messwerte ebenfalls den Netzwerkverkehr verringern.
